use std::process::exit;

use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum LootSpec<T: AsRef<str>> {
    /// Asset specifier
    Item(T),
    /// Asset specifier, lower range, upper range
    ItemQuantity(T, u32, u32),
    /// Loot table
    LootTable(T),
    /// No loot given
    Nothing,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Loot {
    loot: Vec<(f32, LootSpec<String>)>,
}
impl Default for Loot {
    fn default() -> Self {
        Self {
            loot: vec![(0.0, LootSpec::Nothing)],
        }
    }
}
#[derive(Clone, Debug)]
pub struct DungeonsLoot<'a> {
    dungeon_level: &'a str,
    loots: (Loot, reqwest::Url),
}
impl<'a> DungeonsLoot<'a> {
    pub fn new(url: &'a str, dungeon_level: &'a str) -> DungeonsLoot<'a> {
        DungeonsLoot {
            dungeon_level,
            loots: (
                Loot::default(),
                match reqwest::Url::parse(url) {
                    Ok(it) => it,
                    Err(err) => panic!("Url 无法解析. Err: {}", err),
                },
            ),
        }
    }
    async fn format(self) {
        println!("=====正在解析**{}**地牢中...======", self.dungeon_level);
        for loottable in self.clone().request_parse_loots().await.unwrap().iter() {
            println!("{:=^80}", format!("{}", loottable.0));
            let mut weight_sum = 0.0;

            for l in loottable.1.loot.iter() {
                weight_sum += l.0;
            }
            for loot in loottable.1.loot.iter() {
                // 单个物品权重/(所有物品权重的和*100) = 掉率
                println!(
                    "{}{}{}",
                    format!("{:<15}", format!("掉落权重: {}", loot.0)),
                    format!(
                        "{:<20}",
                        format!("掉落概率——{:.2}%", (loot.0 / weight_sum) * 100.0)
                    ),
                    format!(
                        "{:<50}",
                        format!(
                            "战利品: {}",
                            match loot.1.clone() {
                                LootSpec::Item(item) => format!("路径: {}", item),
                                LootSpec::ItemQuantity(item, min, max) =>
                                    format!("数量: {}-{} 路径: {}", min, max, item),
                                LootSpec::LootTable(lt) => format!("指向另一个路径: {}", lt),
                                LootSpec::Nothing => "啥都没有".to_string(),
                            }
                        )
                    )
                );
            }
        }
    }
    //返回单个战利品
    async fn parse_loot(self, loot_name: &str, loot_url: &str) -> Result<Loot, reqwest::Error> {
        println!("解析 **{}** 的战利品中...", loot_name);

        let resp = reqwest::Client::new()
            .get(loot_url.clone())
            .header("User-Agent", "Rust")
            .send()
            .await?;

        let resp_json: serde_json::Value = serde_json::from_str(&resp.text().await?).unwrap();

        let loot_content_need_decode = resp_json["content"]
            .to_string()
            .replace("\\n", "")
            .replace("\"", "");

        // println!("loot_content_need_decode: {}", loot_content_need_decode);
        let loot_content = base64::decode(loot_content_need_decode).unwrap();
        // println!("decode: {:#?}", loot_content);
        let loot_content = format!("(loot: {})", std::str::from_utf8(&loot_content).unwrap());
        // println!("loot content: {}", loot_content);
        let loot: Loot = ron::de::from_str(loot_content.as_str()).unwrap();
        // println!("ENTITY-NAME: {}\nENTITY-STRUCT: {:?}", loot_name, loot);
        Ok(loot)
    }

    // 返回某个地牢等级中的所有战利品
    pub async fn request_parse_loots(self) -> Result<Vec<(String, Loot)>, reqwest::Error> {
        let mut loots: Vec<(String, Loot)> = vec![];

        let resp = reqwest::Client::new()
            .get(self.loots.1.clone())
            .header("User-Agent", "Rust")
            .send()
            .await?;
        println!("获取战利品列表中...状态: {}", resp.status());

        let resp_json: serde_json::Value = serde_json::from_str(&resp.text().await?).unwrap();
        // println!("战力品列表解析: {}", resp_json);

        for loot in resp_json.as_array().unwrap() {
            let loot_name = loot["name"].as_str().unwrap().to_owned();
            let loot_url = loot["url"].as_str().unwrap().to_owned();
            loots.push((
                loot_name.clone(),
                self.clone()
                    .parse_loot(&loot_name, &loot_url)
                    .await
                    .unwrap(),
            ));
        }
        Ok(loots)
    }
}

pub async fn run() {
    let mut dungeons_loots: Vec<DungeonsLoot> = Vec::new();

    dungeons_loots.push(DungeonsLoot::new("https://api.github.com/repos/EvanMeek/veloren-wecw-assets/contents/common/loot_tables/dungeon/tier-0", "tier-0"));
    dungeons_loots.push(DungeonsLoot::new("https://api.github.com/repos/EvanMeek/veloren-wecw-assets/contents/common/loot_tables/dungeon/tier-1", "tier-1"));
    dungeons_loots.push(DungeonsLoot::new("https://api.github.com/repos/EvanMeek/veloren-wecw-assets/contents/common/loot_tables/dungeon/tier-2", "tier-2"));
    dungeons_loots.push(DungeonsLoot::new("https://api.github.com/repos/EvanMeek/veloren-wecw-assets/contents/common/loot_tables/dungeon/tier-3", "tier-3"));
    dungeons_loots.push(DungeonsLoot::new("https://api.github.com/repos/EvanMeek/veloren-wecw-assets/contents/common/loot_tables/dungeon/tier-4", "tier-4"));
    dungeons_loots.push(DungeonsLoot::new("https://api.github.com/repos/EvanMeek/veloren-wecw-assets/contents/common/loot_tables/dungeon/tier-5", "tier-5"));
    dungeons_loots.push(DungeonsLoot::new("https://api.github.com/repos/EvanMeek/veloren-wecw-assets/contents/common/loot_tables/dungeon/wildboss", "wildboss"));

    // dungeons_loots.get(0).unwrap().clone().format().await;
    loop {
        println!("此工具用于查看Veloren-WECW服务器的地牢战利品列表。\n注意事项: 不要频繁使用，否则会因为请求次数过多而无法使用。\n下面请根据提示输入数字选项，别加句点，别瞎鸡巴写。\n{}",format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            "1. T1", "2. T2", "3. T3", "4. T4", "5. T5", "6. T6", "7. WildBoss", "9. 全选", "0. 退出"
        ));
        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();
        match choice {
            "1" => dungeons_loots.get(0).unwrap().clone().format().await,
            "2" => dungeons_loots.get(1).unwrap().clone().format().await,
            "3" => dungeons_loots.get(2).unwrap().clone().format().await,
            "4" => dungeons_loots.get(3).unwrap().clone().format().await,
            "5" => dungeons_loots.get(4).unwrap().clone().format().await,
            "6" => dungeons_loots.get(5).unwrap().clone().format().await,
            "7" => dungeons_loots.get(6).unwrap().clone().format().await,
            "9" => {
                dungeons_loots.get(0).unwrap().clone().format().await;
                dungeons_loots.get(1).unwrap().clone().format().await;
                dungeons_loots.get(2).unwrap().clone().format().await;
                dungeons_loots.get(3).unwrap().clone().format().await;
                dungeons_loots.get(4).unwrap().clone().format().await;
                dungeons_loots.get(5).unwrap().clone().format().await;
                dungeons_loots.get(6).unwrap().clone().format().await;
            }
            "0" => exit(0),
            _ => {
                println!("请选择正确的数字, 按回车继续。");
                let mut tempbuf = String::new();
                std::io::stdin().read_line(&mut tempbuf).unwrap();
                print!("\x1B[2J\x1B[1;1H");
            }
        }
    }
}
