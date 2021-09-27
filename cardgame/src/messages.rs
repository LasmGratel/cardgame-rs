use crate::{Card, LobbyError, RoomError, GameError};
use serde::{Deserialize, Serialize};
use crate::user::{User, UserId};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
pub enum S2CMessage {
    /// 返回延迟
    Pong,

    /// 更新用户数据
    UpdateData(User),

    /// 成功登入
    // TODO token
    LoggedIn,

    /// 房间列表
    RoomList(Vec<String>),

    /// 成功加入房间
    RoomJoined(String),

    /// 房间相关错误
    RoomErr(RoomError),

    /// 大厅相关错误
    LobbyErr(LobbyError),

    /// 游戏相关错误
    GameErr(GameError),

    /// 游戏开始
    /// 参数为手牌以及地主ID
    GameStarted(Vec<Card>, String),

    /// 轮到谁叫地主
    LandlordMove(String),

    /// 地主人选和地主牌
    LordCards(String, Vec<Card>),

    /// 出牌
    CardsSubmitted(String, Vec<Card>),

    /// 匹配列表信息
    /// 当前在匹配的玩家数量，预期时间，剩余时间
    MatchmakeStatus(u32, Duration, Duration),

    /// 该谁出牌
    Move(String),

    /// 重新比赛投票
    RematchVote(UserId, bool, u32)
}

#[derive(Serialize, Deserialize)]
pub enum C2SMessage {
    /// 出牌
    SubmitCards(Vec<Card>),

    /// 过牌
    Pass,

    ChooseLandlord(bool),
    Ping,
    Login(String),
    QueryRoomList,
    JoinRoom(String),

    /// 加入匹配列表
    Matchmake,

    /// 查询匹配状态
    QueryMatchmake,

    StartGame(String),

    RematchVote(bool)
}
