use core::str;
use std::borrow::{Borrow, BorrowMut};
use std::{cell::RefCell, collections::HashMap, ops::Index, rc::Rc, vec};

use base64::decode;
use reqwest::Url;
use ron::ser::to_string_pretty;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use show_me_the_loot_tables::run;

#[tokio::main]
// 掉率公式
// 单个物品权重/(所有物品权重的和*100) = 掉率
async fn main() {
    run().await;
}
