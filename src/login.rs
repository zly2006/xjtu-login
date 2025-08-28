use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use rsa::pkcs8::DecodePublicKey;
use rsa::rand_core::OsRng;
use rsa::{Pkcs1v15Encrypt, RsaPublicKey};
use scraper::{Html, Selector};
use std::fmt::Display;
use thiserror::Error;

pub static BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/139.0.0.0 Safari/537.36";

#[derive(Debug)]
pub enum Service {
    AiPlatform,
}

impl Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Service::AiPlatform => write!(f, "AI 平台"),
        }
    }
}

#[derive(Debug, Error)]
pub enum LoginError {
    #[error("HTTP request error: {0}")]
    RequestError(reqwest::Error),
    #[error("MFA detect failure: {0:?}")]
    MFADetectFailure(Option<serde_json::Value>),
    #[error("Unexpected redirect on {0} but got status code {1}")]
    ExpectedRedirect(String, StatusCode),
    #[error("Login failed")]
    LoginFailed,
    #[error("Other error: {0}")]
    Other(String),
    // Add other error types as needed
}

/**
 * Truncate a string to a maximum length, appending "... (N truncated)" if it was truncated.
 */
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let mut truncated = s[..max_len].to_string();
        truncated.push_str("... (");
        truncated.push_str(&(s.len() - max_len).to_string());
        truncated.push_str(" truncated)");
        truncated
    }
}

pub fn login(service: Service, username: &str, password: &str) -> Result<Client, LoginError> {
    let client = Client::builder()
        .no_proxy() // 禁用 proxy，防止梯子故障。对于校外用户，我们转而使用webvpn登陆
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::none())
        .user_agent(BROWSER_UA)
        .build()
        .map_err(LoginError::RequestError)?;
    log::info!("Logging in to service: {}", service);
    let login_url = match service {
        Service::AiPlatform => {
            let login_start: serde_json::Value = client
                .post("https://ai.xjtu.edu.cn/api/auth/login")
                .json(&serde_json::json!(  {"SSO":"Oauth","IdpID":"1","RedirectUrl":"/"}))
                .send()
                .map_err(LoginError::RequestError)?
                .json()
                .map_err(LoginError::RequestError)?;
            if let serde_json::Value::Object(obj) = &login_start {
                if let Some(serde_json::Value::String(url)) = obj.get("redirect_uri") {
                    url.clone()
                } else {
                    return Err(LoginError::Other(format!(
                        "No url found in login start response: {}",
                        login_start
                    )));
                }
            } else {
                return Err(LoginError::Other(format!(
                    "Unexpected login start response: {}",
                    login_start
                )));
            }
        }
    };

    fn follow_redirects(
        client: &reqwest::blocking::Client,
        url: &str,
    ) -> Result<Response, LoginError> {
        let mut url = url.to_string();
        for _ in 0..10 {
            let resp = client.get(&url).send().map_err(LoginError::RequestError)?;
            if resp.status() == StatusCode::FOUND {
                if let Some(location) = resp.headers().get("Location") {
                    let location = location
                        .to_str()
                        .map_err(|_| LoginError::ExpectedRedirect(url.clone(), resp.status()))?;
                    log::debug!("Redirect to: {}", location);
                    url = location.to_string();
                } else {
                    return Err(LoginError::ExpectedRedirect(url.clone(), resp.status()));
                }
            } else {
                return Ok(resp);
            }
        }
        Err(LoginError::Other("Too many redirects".to_string()))
    }

    let resp = follow_redirects(&client, &login_url)?;
    let post_endpoint = resp.url().to_string();
    log::info!("Login POST endpoint: {}", post_endpoint);
    let html = resp.text().unwrap();
    let document = Html::parse_document(&html);

    // 2. 创建一个 CSS 选择器来查找元素
    let selector = Selector::parse(r#"input[name="execution"]"#).unwrap();
    let execution = document
        .select(&selector)
        .next()
        .unwrap()
        .attr("value")
        .unwrap();
    let selector = Selector::parse(r#"input[name="submit"]"#).unwrap();
    let submit = document
        .select(&selector)
        .next()
        .unwrap()
        .attr("value")
        .unwrap();
    let form = document.select(&selector).next().unwrap().parent().unwrap();
    let inputs: Vec<_> = form
        .children()
        .filter_map(|c| {
            c.value().as_element().and_then(|e| {
                if e.name() == "input" {
                    Some((e.attr("name").unwrap_or(""), e.attr("value").unwrap_or("")))
                } else {
                    None
                }
            })
        })
        .collect();
    let fp_visitor_id = inputs.iter().find(|p| p.0 == "fpVisitorId").unwrap().1;
    log::info!(
        "execution: {}, submit: {}, fpVisitorId: {}",
        truncate_string(execution, 32),
        submit,
        fp_visitor_id
    );

    // encrypt password
    let public_key = RsaPublicKey::from_public_key_pem(include_str!("XJTU_PublicKey")).unwrap();
    let password_encrypted = format!(
        "__RSA__{}",
        base64::encode(
            public_key
                .encrypt(&mut OsRng, Pkcs1v15Encrypt, password.as_bytes())
                .expect("加密失败")
        )
    );

    // detect
    let resp = client
        .post("https://login.xjtu.edu.cn/cas/mfa/detect")
        .form(&[
            ("username", username),
            ("password", &password_encrypted),
            ("fpVisitorId", fp_visitor_id),
        ])
        .send()
        .map_err(LoginError::RequestError)?;
    log::info!("Detecting MFA, status: {}", resp.status());
    let mfa_state = if let Ok(json) = resp.json() {
        if let serde_json::Value::Object(obj) = json {
            if let Some(serde_json::Value::Object(data)) = obj.get("data") {
                if let Some(serde_json::Value::String(mfa_state)) = data.get("state") {
                    mfa_state.clone()
                } else {
                    return Err(LoginError::MFADetectFailure(Some(
                        serde_json::Value::Object(obj),
                    )));
                }
            } else {
                return Err(LoginError::MFADetectFailure(Some(
                    serde_json::Value::Object(obj),
                )));
            }
        } else {
            return Err(LoginError::MFADetectFailure(Some(json)));
        }
    } else {
        return Err(LoginError::MFADetectFailure(None));
    };

    let resp = client
        .post(post_endpoint)
        .form(&[
            ("username", username),
            ("password", &password_encrypted),
            ("execution", execution),
            ("submit1", "Login1"),
            ("_eventId", "submit"),
            ("geolocation", ""),
            ("trustAgent", ""),
            ("fpVisitorId", fp_visitor_id),
            ("trustAgent", ""),
            ("captcha", ""),
            ("currentMenu", "1"),
            ("failN", "0"),
            ("mfaState", &mfa_state),
        ])
        .send()
        .map_err(LoginError::RequestError)?;
    fn expect_redirect(resp: &Response) -> Result<&str, LoginError> {
        if resp.status() != StatusCode::FOUND {
            return Err(LoginError::ExpectedRedirect(
                resp.url().as_str().to_string(),
                resp.status(),
            ));
        }
        let location = resp
            .headers()
            .get("Location")
            .ok_or_else(|| {
                LoginError::ExpectedRedirect(resp.url().as_str().to_string(), resp.status())
            })?
            .to_str()
            .map_err(|_| {
                LoginError::ExpectedRedirect(resp.url().as_str().to_string(), resp.status())
            })?;
        log::debug!("Redirect to: {}", location);
        Ok(location)
    }
    let resp = client
        .get(expect_redirect(&resp)?)
        .send()
        .map_err(LoginError::RequestError)?;
    let resp = client
        .get(expect_redirect(&resp)?)
        .send()
        .map_err(LoginError::RequestError)?;
    let resp = client
        .get(expect_redirect(&resp)?)
        .send()
        .map_err(LoginError::RequestError)?;
    let resp = client
        .get(expect_redirect(&resp)?)
        .send()
        .map_err(LoginError::RequestError)?;
    let url = expect_redirect(&resp)?;
    if !url.starts_with("/login-success") {
        return Err(LoginError::LoginFailed);
    }

    Ok(client)
}
