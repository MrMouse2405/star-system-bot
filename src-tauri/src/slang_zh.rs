use aho_corasick::{AhoCorasick, MatchKind};
use once_cell::sync::Lazy;

// This preprocessor converts idioms/slang into "Baby Chinese"
// (Simple, literal logic) to prevent M2M100 hallucinations.
static SEMANTIC_FLATTENER: Lazy<(AhoCorasick, Vec<&'static str>)> = Lazy::new(|| {
    let mapping = get_mandarin_slang_dict();

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
pub fn normalize_mandarin_slang(text: &str) -> String {
    let (ac, replacements) = &*SEMANTIC_FLATTENER;
    ac.replace_all(text, replacements)
}

fn get_mandarin_slang_dict() -> Vec<(&'static str, &'static str)> {
    // Ideally, for a large dataset, use a HashMap or a Perfect Hash Function (phf crate).
    // Sticking to Vec as requested for simple iteration.
    let mut map = Vec::new();

    // ==========================================
    // 1. PINYIN ACRONYMS (ZIMU QUAN)
    // ==========================================
    map.push(("xswl", "笑死我了")); // Laughing to death (LMAO)
    map.push(("yyds", "永远的神")); // Eternal God (GOAT)
    map.push(("nsdd", "你说得对")); // You are right (often sarcastic agreement)
    map.push(("zqsg", "真情实感")); // True feelings/Heartfelt
    map.push(("awsl", "啊我死了")); // Ah I'm dead (Reaction to cuteness)
    map.push(("plgg", "漂亮哥哥")); // Pretty boy/Handsome guy
    map.push(("plmm", "漂亮妹妹")); // Pretty girl
    map.push(("bdjw", "不懂就问")); // Just asking/ELI5
    map.push(("dbq", "对不起")); // Sorry
    map.push(("bhys", "不好意思")); // Excuse me/Sorry
    map.push(("sjb", "神经病")); // Crazy/Neurotic (Insult)
    map.push(("pyq", "朋友圈")); // WeChat Moments
    map.push(("xjj", "小姐姐")); // Young lady (polite)
    map.push(("xgg", "小哥哥")); // Young man (polite)
    map.push(("u1s1", "有一说一")); // To be honest
    map.push(("dddd", "懂得都懂")); // Those who know, know
    map.push(("gkd", "搞快点")); // Hurry up
    map.push(("srds", "虽然但是")); // Although... but... (Used to transition topics)
    map.push(("yygq", "阴阳怪气")); // Passive aggressive/Sarcastic
    map.push(("nb", "厉害")); // Awesome (Vulgar origin, now common)
    map.push(("rnb", "真厉害")); // Really awesome
    map.push(("woc", "哎呀")); // Fuck/Damn (Exclamation) -> Formal: Oh my
    map.push(("tm", "他妈")); // Damn it
    map.push(("md", "妈的")); // Damn it
    map.push(("nss", "暖说说")); // Comment on status to boost it
    map.push(("cp", "情侣/搭档")); // Couple/Pairing
    map.push(("be", "悲剧结局")); // Bad Ending
    map.push(("he", "圆满结局")); // Happy Ending
    map.push(("oe", "开放结局")); // Open Ending
    map.push(("kswl", "太甜了")); // "Ke si wo le" (I ship this couple so hard)
    map.push(("ssfd", "瑟瑟发抖")); // Shaking with fear (often mock fear)
    map.push(("zfwb", "转发微博")); // Repost Weibo
    map.push(("gz", "关注")); // Follow
    map.push(("szd", "是真的")); // It's true
    map.push(("wtmxs", "笑死我了")); // Dying of laughter (emphatic)
    map.push(("yysy", "有一说一")); // To be honest
    map.push(("bjd", "不知道")); // Don't know
    map.push(("jms", "姐妹们")); // Sisters/Girls
    map.push(("xdm", "兄弟们")); // Brothers/Guys
    map.push(("lz", "楼主")); // Original Poster (OP)
    map.push(("ky", "没眼色")); // Kill the mood (Contextually inappropriate)
    map.push(("zq", "周期")); // Period/Cycle (Context dependent) OR Zao Qi (Good morning)
    map.push(("py", "朋友")); // Friend (or sometimes 'py' deal = dirty deal)
    map.push(("hh", "哈哈")); // Haha

    // ==========================================
    // 2. NUMBER SLANG (HOMOPHONES)
    // ==========================================
    map.push(("666", "厉害/顺利")); // Awesome/Smooth
    map.push(("520", "我爱你")); // I love you
    map.push(("521", "我愿意")); // I am willing / I love you
    map.push(("1314", "一生一世")); // Forever
    map.push(("886", "再见")); // Bye bye
    map.push(("88", "再见")); // Bye
    map.push(("995", "救救我")); // Save me
    map.push(("4242", "是的是的")); // Yes yes
    map.push(("7456", "气死我了")); // Pissed me off
    map.push(("9494", "就是就是")); // Exactly/Agreed
    map.push(("555", "呜呜呜")); // Crying sound
    map.push(("233", "哈哈")); // Laughter
    map.push(("996", "高强度工作")); // 9am-9pm, 6 days work culture
    map.push(("007", "无休工作")); // 24/7 work culture
    map.push(("250", "傻瓜")); // Idiot/Simpleton
    map.push(("3q", "谢谢")); // Thank you
    map.push(("530", "我想你")); // I miss you
    map.push(("065", "原谅我")); // Forgive me
    map.push(("58", "晚安")); // Goodnight
    map.push(("484", "是不是")); // Is it or not?
    map.push(("1920", "依旧爱你")); // Still love you
    map.push(("987", "对不起")); // Sorry
    map.push(("87", "白痴")); // Idiot (Taiwanese origin)
    map.push(("1", "收到/赞同")); // "Received" or "Yes" (Common in gaming/chats)

    // ==========================================
    // 3. COMMON INTERNET CHARACTERS (HANZI)
    // ==========================================
    map.push(("亲", "顾客/亲爱的")); // Dear (Customer service/Taobao)
    map.push(("萌", "可爱")); // Cute (Moe)
    map.push(("囧", "尴尬/无奈")); // Awkward/Embarrassed
    map.push(("赞", "好/支持")); // Like/Good
    map.push(("粉", "粉丝")); // Fan
    map.push(("黑", "批评者")); // Anti-fan/Hater
    map.push(("吹", "吹捧")); // Hype/Boast
    map.push(("水", "灌水/敷衍")); // Spam/Low effort
    map.push(("雷", "震惊/扫兴")); // Shocking/Minefield
    map.push(("坑", "陷阱/劣质")); // Trap/Bad quality/Rip-off
    map.push(("梗", "笑点/话题")); // Meme/Punchline/Trope
    map.push(("草", "哎呀")); // Damn (Censored version of F-word, usually mild frustration)
    map.push(("肝", "拼命工作/游戏")); // Grind (staying up late to work/play)
    map.push(("糊", "过气")); // Flop/Irrelevant (Celebrity career)
    map.push(("怼", "反驳/批评")); // Attack/Retort verbally

    // ==========================================
    // 4. MODERN CONCEPTS & LIFESTYLE
    // ==========================================
    map.push(("躺平", "放弃奋斗")); // Lying flat (doing the bare minimum)
    map.push(("内卷", "恶性竞争")); // Involution (intense, fruitless competition)
    map.push(("摆烂", "破罐破摔")); // Let it rot (giving up completely)
    map.push(("凡尔赛", "低调炫耀")); // Humblebrag
    map.push(("种草", "推荐")); // Recommend (Plant grass)
    map.push(("拔草", "取消购买意愿")); // Decided not to buy (Pull grass)
    map.push(("剁手", "购物")); // Shopping spree
    map.push(("吃瓜", "围观八卦")); // Spectating drama (Eating melon)
    map.push(("瓜", "八卦新闻")); // Gossip/Drama
    map.push(("社畜", "工薪阶层")); // Corporate slave
    map.push(("社恐", "社交恐惧")); // Social anxiety
    map.push(("社牛", "社交达人")); // Social butterfly
    map.push(("爷青回", "唤起回忆")); // My youth is back (Nostalgia)
    map.push(("爷青结", "青春结束")); // My youth is over (Disappointment/Finale)
    map.push(("破防", "深受触动/崩溃")); // Defense broken (Emotional impact)
    map.push(("上头", "着迷/冲动")); // Obsessed/Intoxicated/Impulsive
    map.push(("下头", "扫兴")); // Turn-off/Disenchanted
    map.push(("海王", "花花公子")); // Player (Aquaman - manages many 'fish')
    map.push(("绿茶", "装纯心机")); // Green tea (Fake innocent/Manipulative)
    map.push(("白莲花", "装纯洁")); // White lotus (Fake pure)
    map.push(("键盘侠", "网络喷子")); // Keyboard warrior
    map.push(("柠檬精", "嫉妒的人")); // Jealous person (Sour)
    map.push(("酸", "嫉妒")); // Jealous (Sour)
    map.push(("实锤", "确凿证据")); // Concrete evidence
    map.push(("划水", "偷懒")); // Slacking off
    map.push(("摸鱼", "偷懒")); // Slacking off (Fishing at work)
    map.push(("锦鲤", "好运象征")); // Good luck icon
    map.push(("硬核", "强悍/专业")); // Hardcore
    map.push(("佛系", "随缘/不争")); // Buddhist style (Chill/Let it be)
    map.push(("C位", "核心位置")); // Center position
    map.push(("打call", "支持/加油")); // Cheering for
    map.push(("pick", "选择/喜欢")); // Choose/Support
    map.push(("双标", "双重标准")); // Double standards
    map.push(("真香", "由于喜欢而改变立场")); // "So good" (Eating words/Converting to like something)

    // ==========================================
    // 5. GAMING & SUB-CULTURE
    // ==========================================
    map.push(("老六", "阴险的人/伏地魔")); // Camper/Sneaky player
    map.push(("送人头", "故意送死")); // Feeding (in games)
    map.push(("带飞", "带领获胜")); // Carry (the team)
    map.push(("落地成盒", "秒败")); // Die immediately (PUBG slang)
    map.push(("非酋", "运气不好的人")); // Person with bad luck (Gacha games)
    map.push(("欧皇", "运气极好的人")); // Person with great luck (Gacha games)
    map.push(("氪金", "充值/花钱")); // Pay money (Microtransactions)
    map.push(("肝帝", "极度努力的玩家")); // Hardcore grinder
    map.push(("二次元", "动漫游文化")); // ACG Subculture (2D World)
    map.push(("现充", "现实生活充实者")); // Normie (Real life fulfilled)

    // ==========================================
    // 6. PHONETIC & MEME SLANG (SOUND-ALIKES)
    // ==========================================
    map.push(("集美", "姐妹")); // Sisters (Pronunciation slur)
    map.push(("甚至", "甚至")); // Shenzhi (Often typed as shenshen)
    map.push(("栓Q", "谢谢/无语")); // Thank you (often sarcastic) - From "Thank you"
    map.push(("芭比Q", "完了")); // Finished/Screwed - From "BBQ"
    map.push(("绝绝子", "太棒了")); // Amazing (often used ironically now)
    map.push(("无语子", "无语")); // Speechless
    map.push(("耗子尾汁", "好自为之")); // Look out for yourself (Meme from Ma Baoguo)
    map.push(("蓝瘦香菇", "难受想哭")); // Sad and want to cry (Dialect meme)
    map.push(("雨女无瓜", "与你无关")); // None of your business (Dialect meme)
    map.push(("这就触及到我的知识盲区了", "我不知道")); // That's a blind spot in my knowledge (I don't know)
    map.push(("小丑", "自作多情的人")); // Clown (Simp/Fool)

    // ==========================================
    // 7. NEWER TRENDS (2023-2024)
    // ==========================================
    map.push(("显眼包", "爱出风头/丢人可爱")); // Attention seeker/Goofball
    map.push(("脆皮", "体质差")); // Fragile/Crispy (Young people with bad health)
    map.push(("特种兵", "高强度旅游/活动")); // Special forces (Intense travel schedule)
    map.push(("泰裤辣", "太酷了")); // Too cool (Meme pronunciation)
    map.push(("尊嘟假嘟", "真的假的")); // For real? (Cute talk)
    map.push(("哈基米", "萌宠")); // Cute pet (From "Hachimimi" song)
    map.push(("纯爱战神", "专一的人")); // Loyal lover
    map.push(("服了", "无奈")); // I give up/Unbelievable

    // ==========================================
    // 8. VULGAR SLANG & SWEAR WORDS (The "Ma" & "B" Families)
    // ==========================================
    // "B" Family (Genitalia references used as insults/intensifiers)
    map.push(("sb", "傻逼")); // Stupid c*** (Idiot/Moron) - Extremely common
    map.push(("dsb", "大傻逼")); // Big idiot
    map.push(("jb", "鸡巴")); // P*nis (Often used as "Trash" or intensifier like "fucking")
    map.push(("nb", "牛逼")); // Cow p*ssy (Means: Awesome/Badass) - Vulgar but positive
    map.push(("lb", "老逼")); // Old c*** (Insult for older people)
    map.push(("lowb", "低端/没品")); // Low class/Trashy person
    map.push(("fw", "废物")); // Waste/Loser
    map.push(("nt", "脑瘫")); // Brain damage (Retard)
    map.push(("nc", "脑残")); // Brain dead
    map.push(("zz", "智障")); // Retard (Intellectually disabled)
    map.push(("2b", "二逼")); // Idiot (250 + Stupid)
    map.push(("shabi", "傻逼")); // Idiot (Pinyin full)
    map.push(("bitch", "婊子")); // Bitch (Loan word usage)
    map.push(("lj", "垃圾")); // Trash/Garbage

    // "Ma" Family (Mother insults)
    map.push(("tmd", "他妈的")); // Damn it / His mother's
    map.push(("md", "妈的")); // Damn it
    map.push(("tm", "他妈")); // Damn
    map.push(("nm", "你妈")); // Your mom
    map.push(("cnm", "操你妈")); // F*** your mom
    map.push(("nmb", "你妈逼")); // Your mom's c***
    map.push(("nmsl", "你妈死了")); // Your mom died (Extremely offensive, common in gaming)
    map.push(("wdnmd", "我透你妈")); // I f*** your mom (CS:GO streamer slang)
    map.push(("wnm", "我去你妈")); // Go f*** your mom
    map.push(("mlgb", "马勒戈壁")); // Mother's c*** (Phonetic evasion using alpaca meme)
    map.push(("qnmd", "去你妈的")); // Go f*** your mom / Get out of here

    // Action/Exclamation Vulgarities
    map.push(("woc", "我操")); // Holy sh** / F***
    map.push(("wc", "我操")); // Holy sh** / F***
    map.push(("kao", "靠")); // Damn/Shoot (Softer version of Cao)
    map.push(("ri", "日")); // F*** (Sun)
    map.push(("gun", "滚")); // Get lost / F*** off
    map.push(("gwn", "滚")); // Get lost (Typo/variant)
    map.push(("qnmd", "去你妈的")); // Screw you / F*** off
    map.push(("yp", "约炮")); // Booty call / Hook up
    map.push(("pyjy", "屁眼交易")); // Dirty deal (An*l trade) - Meme for corruption/backdoor deals

    // ==========================================
    // 9. HOSTILE INTERNET SLANG
    // ==========================================
    map.push(("gzn", "郭楠")); // "Guo Nan" (Despectful term for Chinese men)
    map.push(("xn", "仙女")); // Fairy (Sarcastic term for entitled women)
    map.push(("xxn", "小仙女")); // Little Fairy (Sarcastic term for "woke" or entitled women)
    map.push(("nmsl", "你妈死了")); // (Repeated for emphasis)
    map.push(("4000+", "死妈")); // 4000+ (Meme implying someone has no mother)
    map.push(("hsbd", "胡说八道")); // Nonsense / Bullsh**
    map.push(("ntr", "被戴绿帽")); // Cuckold (Netorare)
    map.push(("lz", "老子")); // I/Me (Arrogant: "I, your father")
    map.push(("ye", "爷")); // I/Me (Arrogant: "Grandpa")
    map.push(("xswl", "笑死我了")); // LMAO (Can be mocking)
    map.push(("yygq", "阴阳怪气")); // Sarcastic/Passive-aggressive

    map
}
