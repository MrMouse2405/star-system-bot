use aho_corasick::{AhoCorasick, MatchKind};
use once_cell::sync::Lazy;

// This preprocessor converts idioms/slang into "Baby Chinese"
// (Simple, literal logic) to prevent M2M100 hallucinations.
static SEMANTIC_FLATTENER: Lazy<(AhoCorasick, Vec<&'static str>)> = Lazy::new(|| {
    let mapping = get_japanese_slang_dict();

    let mut patterns = Vec::new();
    let mut replacements = Vec::new();

    for (slang, simple) in mapping {
        patterns.push(slang);
        replacements.push(simple);
    }

    // LeftmostLongest is crucial for "这波" vs "这波操作"
    let ac = AhoCorasick::builder()
        .match_kind(MatchKind::LeftmostLongest)
        .build(&patterns)
        .expect("Failed to build Automaton");

    (ac, replacements)
});

/// Preprocesses Mandarin text by replacing slang with formal text
/// suitable for translation models like M2M100.
pub fn normalize_japanese_slang(text: &str) -> String {
    let (ac, replacements) = &*SEMANTIC_FLATTENER;
    ac.replace_all(text, replacements)
}

fn get_japanese_slang_dict() -> Vec<(&'static str, &'static str)> {
    let mut map = Vec::new();

    // ==========================================
    // 1. ALPHABET & ROMAJI SLANG (NET SLANG)
    // ==========================================
    map.push(("w", "笑")); // Lol (Warai)
    map.push(("www", "大爆笑")); // Lololol (Lots of grass)
    map.push(("jk", "女子高生")); // High school girl OR "Joudan wa Kao dake" (Joke) context dependent
    map.push(("dk", "男子高校生")); // High school boy
    map.push(("ky", "空気が読めない")); // Can't read the room
    map.push(("kwsk", "詳しく")); // Details please (Kuwashiku)
    map.push(("wktk", "ワクワク")); // Excited (Waku waku teka teka)
    map.push(("ggrks", "ググれ")); // Google it yourself (Googure kasu)
    map.push(("ks", "カス")); // Trash/Scum (Kasu)
    map.push(("gkbr", "ガクガクブルブル")); // Trembling with fear
    map.push(("ng", "駄目")); // No good / Bad
    map.push(("gj", "よくやった")); // Good Job
    map.push(("pk", "プレイヤーキル")); // Player Kill
    map.push(("ry", "以下省略")); // Abbreviated/Etc (Ryakusu)
    map.push(("now", "現在")); // Doing it now (e.g., Ramen now)
    map.push(("nau", "現在")); // Doing it now (Kana version)
    map.push(("wazu", "でした")); // Was (Did it previously)
    map.push(("will", "する予定")); // Will do

    // ==========================================
    // 2. KANJI & KATAKANA NET SLANG
    // ==========================================
    map.push(("草", "面白い")); // Lol (Kusa - looks like 'www')
    map.push(("草生える", "面白い")); // It's funny (Grass grows)
    map.push(("乙", "お疲れ様")); // Good work/Thanks (Otsu)
    map.push(("うぽつ", "アップロードお疲れ様")); // Thanks for upload (Video sites)
    map.push(("おめ", "おめでとう")); // Congrats
    map.push(("あり", "ありがとう")); // Thanks
    map.push(("よろ", "よろしく")); // Please/Nice to meet you
    map.push(("乙カレー", "お疲れ様")); // Good work (Pun on Curry)
    map.push(("鯖", "サーバー")); // Server (Saba - Mackerel pun)
    map.push(("垢", "アカウント")); // Account (Aka - Dirt pun)
    map.push(("鍵垢", "非公開アカウント")); // Private account (Locked)
    map.push(("本垢", "メインアカウント")); // Main account
    map.push(("裏垢", "サブアカウント")); // Alt/Secret account
    map.push(("誰得", "誰が得するの")); // Who benefits from this? (Useless info)
    map.push(("情弱", "情報弱者")); // Uninformed person (Tech illiterate)
    map.push(("壁打ち", "独り言")); // Talking to oneself (Wall hitting)
    map.push(("ROM", "見るだけ")); // Lurker (Read Only Member)
    map.push(("ノ", "挙手")); // *Raises hand* / Hi / I'll go
    map.push(("888", "拍手")); // Clapping (Pachi pachi)

    // ==========================================
    // 3. YOUTH & SOCIAL MEDIA SLANG (JK SLANG)
    // ==========================================
    map.push(("ぴえん", "悲しい")); // Sad/Crying (Cutesy)
    map.push(("それな", "その通り")); // Exactly/Agreed (Big mood)
    map.push(("わかりみ", "共感")); // I understand/empathize
    map.push(("つらたん", "辛い")); // Painful/Hard (Cute version)
    map.push(("やばたにえん", "やばい")); // Crazy/Bad (Pun on Ochazuke brand)
    map.push(("ワンチャン", "可能性がある")); // One chance (Maybe/Possibly)
    map.push(("あーね", "あーなるほど")); // Ah, I see
    map.push(("とりま", "とりあえず")); // For now/Anyway
    map.push(("すこ", "好き")); // Like/Love (Typo turned slang)
    map.push(("尊い", "最高")); // Precious/Divine (Obsessed with character)
    map.push(("エモい", "感動的")); // Emotional/Nostalgic (Emo)
    map.push(("リア充", "充実している人")); // Normie (Fulfilled in real life)
    map.push(("陽キャ", "明るい人")); // Extrovert/Cheerful character
    map.push(("陰キャ", "暗い人")); // Introvert/Gloomy character
    map.push(("パリピ", "パーティー好き")); // Party people
    map.push(("じわる", "じわじわ笑える")); // Slowly becoming funny
    map.push(("バズる", "流行る")); // To go viral (Buzz)
    map.push(("映える", "見栄えが良い")); // Instagrammable (Haeru)
    map.push(("盛れる", "可愛く見える")); // Looking good (filtered/makeup)
    map.push(("推し", "好きな人")); // Fave/Bias (Idol/Character support)
    map.push(("沼", "没頭")); // Swamp (Deeply into a fandom)

    // ==========================================
    // 4. GAMING SLANG (FPS/MMO)
    // ==========================================
    map.push(("ラグい", "遅延がある")); // Laggy
    map.push(("野良", "知らない人")); // Randoms (Matchmaking strangers)
    map.push(("凸", "突撃")); // Rush/Charge (or call/visit)
    map.push(("エイム", "照準")); // Aim
    map.push(("キルレ", "キルレート")); // K/D Ratio
    map.push(("芋", "キャンプする人")); // Camper (Potato - grows roots in one spot)
    map.push(("砂", "スナイパー")); // Sniper (Suna - Sand/Sniper sound)
    map.push(("確キル", "とどめ")); // Confirm kill / Finish off
    map.push(("蘇生", "復活させる")); // Revive/Res
    map.push(("バフ", "強化")); // Buff
    map.push(("ナーフ", "弱体化")); // Nerf
    map.push(("チーター", "不正行為者")); // Cheater
    map.push(("スマーフ", "初心者狩り")); // Smurf
    map.push(("姫プ", "守られるプレイ")); // Princess play (Being protected by simps)
    map.push(("地雷", "下手な人")); // Mine (Bad player/Feeder OR Trigger topic)
    map.push(("トロール", "迷惑行為")); // Troll
    map.push(("沼る", "失敗し続ける")); // Stuck/Failing repeatedly (Bogged down)
    map.push(("ワンパン", "一撃で倒す")); // One punch (One shot kill)
    map.push(("gg", "良い試合だった")); // Good Game
    map.push(("nt", "惜しかった")); // Nice Try

    // ==========================================
    // 5. NEGATIVE / CRITICAL SLANG
    // ==========================================
    map.push(("オワコン", "時代遅れ")); // Ended content (Dead game/trend)
    map.push(("詰んだ", "終わった")); // Checkmate/Screwed/Hopeless
    map.push(("害悪", "迷惑な人")); // Cancer/Toxic person
    map.push(("老害", "迷惑な年長者")); // Boomer/Toxic elder
    map.push(("キッズ", "子供っぽい人")); // Kid/Squeaker
    map.push(("厨二病", "自意識過剰")); // 8th grader syndrome (Edgy teen behavior)
    map.push(("炎上", "批判殺到")); // Flaming/Controversy
    map.push(("ディスる", "批判する")); // Diss/Criticize
    map.push(("マウント", "優位を誇示")); // Mount (One-upmanship/Flexing)
    map.push(("クソゲー", "悪いゲーム")); // Kusoge (Shitty game)

    // ==========================================
    // 6. THE "KUSO" FAMILY (Shit/F***)
    // ==========================================
    map.push(("kuso", "クソ")); // Shit/F*** (Generic intensifier or insult)
    map.push(("ks", "カス")); // Scum/Trash (Kasu) - Very common in gaming
    map.push(("ksg", "クソガキ")); // Shitty brat (Kuso Gaki)
    map.push(("ksjj", "クソジジイ")); // Shitty old man/fart
    map.push(("ksbba", "クソババア")); // Shitty old hag
    map.push(("kusowarota", "クソワロタ")); // Laughed like sh** (LMAO)
    map.push(("gomikas", "ゴミカス")); // Trash scum (Combo insult)
    map.push(("hage", "ハゲ")); // Baldy (Common low-grade insult)

    // ==========================================
    // 7. THE "DEATH" FAMILY (Hostile)
    // ==========================================
    map.push(("shine", "死ね")); // Die (Imperative)
    map.push(("shi ne", "死ね")); // Die (Spaced to avoid filters)
    map.push(("4ne", "死ね")); // Die (4=Shi)
    map.push(("氏ね", "死ね")); // Die (Net slang using "Clan/Mr" Kanji to bypass filters)
                                // CRITICAL: M2M100 will translate 氏ね as "Mr." or "Clan" without this mapping.
    map.push(("tahine", "死ね")); // Die (Typing bypass "ta-hi-ne" looks like "shi-ne" handwritten)
    map.push(("satsu", "殺す")); // Kill (Satsugai)
    map.push(("564", "殺し")); // Kill (5=Go, 6=Ro, 4=Shi -> Goroshi)

    // ==========================================
    // 8. CHARACTER ATTACKS (Stupid/Crazy/Ugly)
    // ==========================================
    map.push(("baka", "馬鹿")); // Idiot (Standard)
    map.push(("aho", "阿呆")); // Idiot (Kansai style)
    map.push(("bk", "馬鹿")); // Idiot (Abbr)
    map.push(("kichi", "気違い")); // Crazy/Lunatic (Abbr of Kichigai)
    map.push(("kitsch", "気違い")); // Crazy (Kichigai)
    map.push(("menhera", "精神不安定")); // Mental health/Psycho (Derogatory)
    map.push(("ikezuman", "性格が悪い")); // Nasty person (Ikezu = mean)
    map.push(("busu", "ブス")); // Ugly woman (Highly offensive)
    map.push(("bba", "ババア")); // Hag/Old woman
    map.push(("jji", "ジジイ")); // Old fart
    map.push(("doutei", "童貞")); // Virgin (Used as insult for men: Incel-ish connotation)
    map.push(("yariman", "ヤリマン")); // Slut (Woman who does it a lot)
    map.push(("bitch", "尻軽女")); // Slut (Note: In JP, 'bitch' specifically means loose woman, not just mean)
    map.push(("dqn", "非常識な人")); // Delinquent/Trashy person (DQN)

    // ==========================================
    // 9. AGGRESSIVE PRONOUNS (Fighting words)
    // ==========================================
    // M2M100 often translates these as simple "You", missing the aggression.
    // Qwen can use the context to add "You bastard" or "You pos".
    map.push(("temee", "てめえ")); // You (Bastard)
    map.push(("omae", "お前")); // You (Aggressive if not close friends)
    map.push(("kisama", "貴様")); // You (Archaic/Anime villain style - extremely hostile)
    map.push(("koitsu", "こいつ")); // This guy (Disrespectful)
    map.push(("aitsu", "あいつ")); // That guy (Disrespectful)

    // ==========================================
    // 10. DISMISSIVE / ANNOYANCE
    // ==========================================
    map.push(("uzai", "うざい")); // Annoying/Pain in the ass
    map.push(("uza", "うざい")); // Annoying
    map.push(("kimo", "気持ち悪い")); // Gross/Creepy
    map.push(("kimoi", "気持ち悪い")); // Gross/Creepy
    map.push(("tsukaen", "使えない")); // Useless (Can't use)
    map.push(("fuzakenna", "ふざけるな")); // Don't f*** around / Stop bullsh****ing
    map.push(("damare", "黙れ")); // Shut up
    map.push(("urusai", "うるさい")); // Shut up / Noisy
    map.push(("dasai", "ダサい")); // Lame/Uncool
    map.push(("das", "ダサい")); // Lame (Abbr)

    map
}
