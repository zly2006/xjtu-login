use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// 性别限制类型
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum GenderLimit {
    /// 不限制性别
    #[serde(rename = "0")]
    None,
    /// 仅限男生
    #[serde(rename = "1")]
    Male,
    /// 仅限女生
    #[serde(rename = "2")]
    Female,
}

impl Display for GenderLimit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GenderLimit::None => write!(f, "不限"),
            GenderLimit::Male => write!(f, "仅限男生"),
            GenderLimit::Female => write!(f, "仅限女生"),
        }
    }
}

/// 课程会话，用于管理用户登录状态和课程操作
pub struct CourseSession {
    /// 学号
    pub number: String,
    /// 名字
    pub name: String,
    /// 客户端
    pub client: Client,
    token: String,
}

impl CourseSession {
    pub async fn fron_client(client: Client) -> Option<Self> {
        let json = client
            .get("https://xkfw.xjtu.edu.cn/xsxkapp/sys/xsxkapp/student/register.do")
            .send()
            .await
            .ok()?
            .json::<serde_json::Value>()
            .await
            .ok()?;
        Some(Self {
            number: json["data"]["number"].as_str()?.to_string(),
            name: json["data"]["name"].as_str()?.to_string(),
            token: json["data"]["token"].as_str()?.to_string(),
            client,
        })
    }
}

/// 选课批次信息
#[derive(Deserialize, Debug)]
pub struct Batch {
    #[serde(rename = "batchType")]
    pub batch_type: String,
    #[serde(rename = "beginTime")]
    pub begin_time: String,
    pub code: String,
    #[serde(rename = "endTime")]
    pub end_time: String,
    pub name: String,
    #[serde(rename = "schoolTerm")]
    pub school_term: String,
    #[serde(rename = "schoolTermName")]
    pub school_term_name: String,
    /// 选课策略代码
    #[serde(rename = "tacticCode")]
    pub tactic_code: String,
    /// 选课策略名称 e.g. 可选可退
    #[serde(rename = "tacticName")]
    pub tactic_name: String,
    /// 选课类型代码
    #[serde(rename = "typeCode")]
    pub type_code: String,
    /// 选课类型名称 e.g. 正选
    #[serde(rename = "typeName")]
    pub type_name: String,
    /// 选课周次范围 e.g. 1-16周
    #[serde(rename = "weekRange")]
    pub week_range: String,
}

/// 获取选课批次
pub async fn get_batch_list(client: &Client) -> Result<Vec<Batch>, reqwest::Error> {
    let resp = client
        .get("https://xkfw.xjtu.edu.cn/xsxkapp/sys/xsxkapp/elective/batch.do")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let batches = resp["dataList"].clone();
    let batches: Vec<Batch> = serde_json::from_value(batches).unwrap_or_default();
    Ok(batches)
}

/// 选课类型，按照网页顺序
#[derive(Deserialize, Serialize, Debug)]
pub enum CourseType {
    /// 主修推荐课程
    TJKC,
    /// 方案内跨年级（其他）课程
    FANKC,
    /// 方案内跨专业（其他）课程
    FAWKC,
    /// 基础通识类（核心/选修）
    XGXK,
    /// 重修课程
    CXKC,
    /// 体育课程
    TYKC,
    /// 辅修课程
    FXKC,
    /// 全校课表查询
    QXKC,
}

/*
{"dataList":[{"departmentName":"经济与金融学院","courseNatureName":"选修","retakeType":null,"campusName":null,"selected":true,"tcList":[{"alternateStatus":null,"isCanAlternate":"0","capacityOfMale":"0","capacityOfFemale":"0","numberOfMale":"11","numberOfFemale":"4","extInfo":null,"conflictDesc":null,"hasTest":"0","testTeachingClassID":null,"isTest":"0","numberOfSelected":"16","limitGender":"0","isFull":"0","isConflict":"0","sportName":null,"courseIndex":"01","courseNatureName":null,"departmentName":null,"courseTypeName":null,"courseNumber":"FINA520919","teachingClassID":"202520261FINA52091901","classCapacity":"30","numberOfFirstVolunteer":"16","isChoose":"1","chooseVolunteer":null,"teacherName":"何建奎","teachingPlace":"1-16周 星期一 1-2节 主楼C-206,13-16周 星期四 7-8节 主楼C-206","teachingMethod":"面授讲课","capacitySuffix":"","recommendSchoolClass":"国贸2301","retakeType":null,"retakeTypeDetail":null,"courseUrl":null,"isLimitKind":"1","inQuene":null,"operationType":"1","courseFlag":null,"needBook":null,"hasBook":"0","campus":"1","campusName":"兴庆校区","teachCampus":"***"}],"courseNumber":"FINA520919","courseName":"国际结算","number":1,"type":"97","typeName":"专业选修课程","hours":"40","credit":"2.5","retakeTypeDetail":null,"courseUrl":null,"majorFlag":"主","courseFlag":null,"wid":null}],"totalCount":1,"keyExpired":null,"code":"1","msg":"查询数据成功","timestamp":"32","map":null}

 */
/// 课程基本信息
#[derive(Deserialize)]
pub struct CourseInfo {
    /// 院系名称
    #[serde(rename = "departmentName")]
    pub department_name: String,
    /// 课程性质名称（如：必修、选修等）
    #[serde(rename = "courseNatureName")]
    pub course_nature_name: String,
    /// 是否已选中该课程
    pub selected: bool,
    /// 该课程下的教学班列表
    #[serde(rename = "tcList")]
    pub tc_list: Vec<TeachingClass>,
    /// 课程号（唯一标识）
    #[serde(rename = "courseNumber")]
    pub course_number: String,
    /// 课程名称
    #[serde(rename = "courseName", default)]
    pub course_name: String,
    /// 课程类型代码
    #[serde(rename = "type")]
    pub type_code: String,
    /// 课程类型名称（如：专业必修课程、专业选修课程等）
    #[serde(rename = "typeName")]
    pub type_name: String,
    /// 课程总学时
    pub hours: String,
    /// 课程学分
    pub credit: String,
    /// 主修标志（主修/辅修）
    #[serde(rename = "majorFlag")]
    pub major_flag: String,
}

/// 教学班信息
#[derive(Deserialize)]
pub struct TeachingClass {
    /// 课程号
    #[serde(rename = "courseNumber")]
    pub course_number: String,
    /// 教学班唯一标识ID
    #[serde(rename = "teachingClassID")]
    pub teaching_class_id: String,
    /// 任课教师姓名
    #[serde(rename = "teacherName")]
    pub teacher_name: String,
    /// 上课时间和地点详细信息
    #[serde(rename = "teachingPlace")]
    pub teaching_place: String,
    /// 教学班最大容量
    #[serde(rename = "classCapacity")]
    pub class_capacity: String,
    /// 当前已选课人数
    #[serde(rename = "numberOfSelected")]
    pub number_of_selected: String,
    /// 性别限制
    #[serde(rename = "limitGender")]
    pub limit_gender: GenderLimit,
    /// 当前用户是否已选择此教学班
    #[serde(rename = "isChoose", deserialize_with = "deserialize_bool_from_string")]
    pub is_choose: bool,
    /// 教学班是否已满员
    #[serde(rename = "isFull", deserialize_with = "deserialize_bool_from_string")]
    pub is_full: bool,
    /// 是否与已选课程时间冲突
    #[serde(rename = "isConflict", deserialize_with = "deserialize_bool_from_string")]
    pub is_conflict: bool,
}

/// 自定义反序列化函数：将字符串"0"/"1"转换为布尔值
fn deserialize_bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s == "1")
}

impl CourseSession {
    /// 列出指定条件的课程
    /// # Arguments
    /// * `batch` - 选课批次
    /// * `course_type` - 课程类型
    /// * `page` - 页码，从0开始
    /// * `query` - 搜索关键词
    /// # Returns
    /// 返回课程信息列表
    pub async fn list_course(
        &self,
        batch: &Batch,
        course_type: CourseType,
        page: u32,
        query: &str,
    ) -> Vec<CourseInfo> {
        let params = serde_json::json!({
            "data": {
                "studentCode": self.number,
                "campus": "1",
                "electiveBatchCode": batch.code,
                "isMajor": "1",
                "teachingClassType": format!("{:?}", course_type),
                "checkConflict": "2",
                "checkCapacity": "2",
                "queryContent": query
            },
            "pageSize": "10",
            "pageNumber": format!("{}", page),
            "order": ""
        });
        let resp = self
            .client
            .post("https://xkfw.xjtu.edu.cn/xsxkapp/sys/xsxkapp/elective/programCourse.do")
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("X-Requested-With", "XMLHttpRequest")
            .header("token", &self.token)
            .body(format!(
                "querySetting={}",
                urlencoding::encode(&params.to_string())
            ))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        let courses = resp["dataList"].clone();
        let courses: Vec<CourseInfo> = serde_json::from_value(courses).unwrap();
        for course in &courses {
            println!("{} - {} - {}", course.course_number, course.course_name, if course.selected { "已选" } else { "未选" });
            for tc in &course.tc_list {
                println!("  {} - {} - {} - {}/{} - {}",
                    tc.teaching_class_id,
                    tc.teacher_name,
                    tc.teaching_place,
                    tc.number_of_selected,
                    tc.class_capacity,
                    if tc.is_choose { "已选" } else { "未选" }
                );
            }
        }
        courses
    }

    /// 取消选课志愿
    /// # Arguments
    /// * `batch` - 选课批次信息
    /// * `class_id` - 教学班ID
    pub async fn delete_volunteer(&self, batch: &Batch, class_id: &str) {
        let params = serde_json::json!({
            "data":{
                "operationType": "2",
                "studentCode": self.number,
                "electiveBatchCode": batch.code,
                "teachingClassId": class_id,
                "isMajor": "1"
            }
        });
        let resp = self
            .client
            .get("https://xkfw.xjtu.edu.cn/xsxkapp/sys/xsxkapp/elective/deleteVolunteer.do")
            .query(&[("deleteParam", params.to_string())])
            .header("X-Requested-With", "XMLHttpRequest")
            .header("token", &self.token)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        println!("{}", resp);
    }

    /// 添加选课志愿
    /// # Arguments
    /// * `batch` - 选课批次信息
    /// * `class_id` - 教学班ID
    /// * `course_type` - 课程类型
    pub async fn add_volunteer(&self, batch: &Batch, class_id: &str, course_type: CourseType) {
        let params = serde_json::json!({
            "data":{
                "operationType": "1",
                "studentCode": self.number,
                "electiveBatchCode": batch.code,
                "teachingClassId": class_id,
                "isMajor": "1",
                "campus": "1",
                "teachingClassType": format!("{:?}", course_type)
            }
        });
        let resp = self
            .client
            .post("https://xkfw.xjtu.edu.cn/xsxkapp/sys/xsxkapp/elective/volunteer.do")
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .body(format!(
                "addParam={}",
                urlencoding::encode(&params.to_string())
            ))
            .header("X-Requested-With", "XMLHttpRequest")
            .header("token", &self.token)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        println!("{}", resp);
    }

    /// 获取教学班容量详细信息
    /// # Arguments
    /// * `class_id` - 教学班ID
    /// # Returns
    /// 返回容量信息结构体
    pub async fn get_capacity(&self, class_id: &str) -> CapacityInfo {
        let resp = self
            .client
            .get("https://xkfw.xjtu.edu.cn/xsxkapp/sys/xsxkapp/elective/teachingclass/capacity.do")
            .query(&[("teachingClassId", class_id), ("capacitySuffix", "")])
            .header("X-Requested-With", "XMLHttpRequest")
            .header("token", &self.token)
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        let number_of_male: u32 = resp["data"]["numberOfMale"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let capacity_of_male: u32 = resp["data"]["capacityOfMale"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let number_of_female: u32 = resp["data"]["numberOfFemale"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let capacity_of_female: u32 = resp["data"]["capacityOfFemale"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let number_of_selected: u32 = resp["data"]["numberOfSelected"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let class_capacity: u32 = resp["data"]["classCapacity"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        CapacityInfo {
            number_of_male,
            capacity_of_male,
            number_of_female,
            capacity_of_female,
            number_of_selected,
            class_capacity,
        }
    }
}

/// 教学班容量详细信息
pub struct CapacityInfo {
    /// 已选男生人数
    pub number_of_male: u32,
    /// 男生容量限制
    pub capacity_of_male: u32,
    /// 已选女生人数
    pub number_of_female: u32,
    /// 女生容量限制
    pub capacity_of_female: u32,
    /// 总已选人数
    pub number_of_selected: u32,
    /// 教学班总容量
    pub class_capacity: u32,
}

impl Display for CapacityInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "男生：{}/{}，女生：{}/{}，总计：{}/{}",
            self.number_of_male,
            self.capacity_of_male,
            self.number_of_female,
            self.capacity_of_female,
            self.number_of_selected,
            self.class_capacity,
        )
    }
}
