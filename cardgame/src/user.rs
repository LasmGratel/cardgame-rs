use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

/**
 * 游戏玩家数据，用于统计积分和用户信息。
 * 当玩家断开连接时销毁。
 */
#[derive(Serialize, Deserialize, Eq, Clone)]
pub struct User {
    /// 玩家名称，必须唯一
    pub id: String,
    // TODO nick: String,

    /// 积分
    pub score: i64,
}

/// 用户标识符
pub type UserId = String;

impl User {
    pub fn new(id: String) -> User {
        User {
            id,
            score: 0
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// 用户状态
pub enum UserState {
    /// 无操作，一般用于大厅
    Idle,

    /// 匹配玩家
    Matchmaking,

    /// 正在游玩，参数为房间名称
    Playing(String)
}

/// 玩家数据管理
/// TODO 数据库
pub struct UserManager {
    /// 用户数据目录
    path: String,

    /// 用户缓存
    cache: HashMap<String, User>
}

impl UserManager {
    pub fn new(path: String) -> UserManager {
        let p = Path::new(&path);
        if !p.is_dir() {
            fs::create_dir(p).expect("Unable to create user directory");
        }

        UserManager {
            path,
            cache: HashMap::new()
        }
    }

    pub fn get_user(&self, id: &String) -> Option<User> {
        if self.cache.contains_key(id) {
            self.cache.get(id).map(|x| x.clone())
        } else {
            let user = self.read_user(id);
            if let Ok(user) = user { // 存在这个文件
                Some(user)
            } else {
                Some(User::new(id.clone()))
            }
        }
    }

    pub fn insert_user(&mut self, id: String, user: User) {
        self.cache.insert(id, user);
        self.write();
    }

    pub fn get_user_mut(&mut self, id: &String) -> &mut User {
        if self.cache.contains_key(id) {
            self.cache.get_mut(id).unwrap()
        } else {
            let user = self.read_user_or_create(id);
            self.cache.insert(id.clone(), user);
            self.cache.get_mut(id).unwrap()
        }
    }

    pub fn write(&self) {
        self.create_dir_if_not_exists();

        let mut counter = 0;
        for user in self.cache.values().into_iter() {
            self.write_user(user);
            counter += 1;
        }
        println!("Saved {} users", counter);
    }

    pub fn read(&mut self) -> Result<(), std::io::Error> {
        self.create_dir_if_not_exists();

        let entries = fs::read_dir(&self.path)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;

        for file in entries.into_iter() {
            let str = fs::read_to_string(file)?;
            let user: User = serde_json::from_str(&str).unwrap();
            self.cache.insert(user.id.clone(), user);
            drop(str);
        }
        Ok(())
    }

    fn create_dir_if_not_exists(&self) {
        let p = Path::new(&self.path);
        if !p.is_dir() {
            fs::create_dir(p).expect("Unable to create user directory");
        }
    }

    fn read_user_or_create(&self, id: &String) -> User {
        if let Ok(user) = self.read_user(id) {
            user
        } else {
            User::new(id.clone())
        }
    }

    /// 从文件读取用户数据
    fn read_user(&self, id: &String) -> Result<User, ()> {
        let path = Path::new(&self.path).join(id);
        let str = fs::read_to_string(path);
        if str.is_ok() {
            serde_json::from_str(&str.unwrap()).map_err(|e| ())
        } else {
            Err(())
        }
    }

    /// 写入文件
    fn write_user(&self, user: &User) {
        let path = Path::new(&self.path).join(&user.id);
        fs::write(path, serde_json::to_string(user).unwrap()).expect("Unable to write user");
    }
}