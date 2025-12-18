use aho_corasick::{AhoCorasick, MatchKind};
use once_cell::sync::Lazy;

// This preprocessor converts idioms/slang into "Baby Chinese"
// (Simple, literal logic) to prevent M2M100 hallucinations.
static SEMANTIC_FLATTENER: Lazy<(AhoCorasick, Vec<&'static str>)> = Lazy::new(|| {
    let mapping = get_french_slang_dict();

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
pub fn normalize_french_slang(text: &str) -> String {
    let (ac, replacements) = &*SEMANTIC_FLATTENER;
    ac.replace_all(text, replacements)
}

fn get_french_slang_dict() -> Vec<(&'static str, &'static str)> {
    let mut map = Vec::new();

    // ==========================================
    // 1. TEXTING ACRONYMS (UNIVERSAL/FRANCE)
    // ==========================================
    map.push(("mdr", "mort de rire")); // LOL (Dying of laughter)
    map.push(("ptdr", "pété de rire")); // LMAO (Farting/Broken with laughter)
    map.push(("xptdr", "explosé de rire")); // ROFL
    map.push(("jpp", "je n'en peux plus")); // I can't even / I'm done
    map.push(("tg", "tais-toi")); // Shut up (Vulgar: Ta gueule)
    map.push(("ftg", "ferme ta gueule")); // Shut the f*** up
    map.push(("pk", "pourquoi")); // Why
    map.push(("pq", "pourquoi")); // Why (or toilet paper, context dependent)
    map.push(("stp", "s'il te plaît")); // Please
    map.push(("svp", "s'il vous plaît")); // Please (Formal)
    map.push(("tkt", "ne t'inquiète pas")); // Don't worry
    map.push(("bsx", "bisous")); // Kisses
    map.push(("bz", "bisous")); // Kisses (Careful: 'baiser' means f***, but 'bz' usually kisses in text)
    map.push(("cc", "coucou")); // Hi/Hey
    map.push(("bjr", "bonjour")); // Hello
    map.push(("sllt", "salut")); // Hi
    map.push(("cv", "ça va")); // How are you?
    map.push(("tfq", "tu fais quoi")); // What are you doing?
    map.push(("koi", "quoi")); // What
    map.push(("ki", "qui")); // Who
    map.push(("auj", "aujourd'hui")); // Today
    map.push(("a+", "à plus tard")); // See you later
    map.push(("osef", "on s'en fiche")); // Who cares / We don't care (Vulgar: On s'en fout)
    map.push(("balek", "je m'en fiche")); // I don't care (Vulgar: Bat les couilles)
    map.push(("oklm", "au calme")); // Chilling / Relaxed
    map.push(("askip", "à ce qu'il parait")); // Apparently / Rumor has it
    map.push(("bg", "beau gosse")); // Handsome guy / Good job
    map.push(("blc", "je m'en fiche")); // I don't care (Bat les couilles)
    map.push(("fdp", "imbécile")); // Son of a b**** (Insult, rarely affectionate)
    map.push(("niques", "parents")); // "Nique ta mere" (Your mom) - deeply offensive usually

    // ==========================================
    // 2. VERLAN (FRANCE - INVERTED SYLLABLES)
    // ==========================================
    map.push(("cimer", "merci")); // Thanks
    map.push(("meuf", "femme")); // Woman/Girl/Girlfriend
    map.push(("keum", "homme")); // Man/Boyfriend (from 'mec')
    map.push(("mec", "homme")); // Guy/Dude
    map.push(("ouf", "fou")); // Crazy
    map.push(("truc de ouf", "incroyable")); // Crazy thing
    map.push(("chelou", "louche")); // Weird/Shady
    map.push(("relou", "lourd")); // Annoying/Heavy
    map.push(("vénère", "énervé")); // Angry
    map.push(("chanmé", "méchant")); // Wicked/Awesome (ironic) or Mean
    map.push(("teuf", "fête")); // Party
    map.push(("pécho", "séduire/attraper")); // To hook up / To catch
    map.push(("reup", "père")); // Father
    map.push(("renoi", "noir")); // Black person
    map.push(("beuh", "herbe")); // Weed (Herbe)
    map.push(("ass", "ça")); // That (Comme ass -> Comme ça)
    map.push(("zarbi", "bizarre")); // Bizarre

    // ==========================================
    // 3. GENERAL FRANCE SLANG
    // ==========================================
    map.push(("wesh", "salut/hé")); // Yo / Hey (Arabic origin)
    map.push(("kiffer", "aimer")); // To like/love
    map.push(("seum", "rancoeur")); // Salty/Bitter (avoir le seum)
    map.push(("thune", "argent")); // Money
    map.push(("fric", "argent")); // Money
    map.push(("balle", "euro")); // Euro (100 balles = 100 euros)
    map.push(("boulot", "travail")); // Work
    map.push(("taffer", "travailler")); // To work
    map.push(("bouffer", "manger")); // To eat
    map.push(("graille", "manger")); // To eat
    map.push(("clope", "cigarette")); // Cigarette
    map.push(("baraque", "maison")); // House
    map.push(("caisse", "voiture")); // Car
    map.push(("flic", "policier")); // Cop
    map.push(("keuf", "policier")); // Cop
    map.push(("boloss", "idiot")); // Loser/Idiot
    map.push(("daron", "père")); // Dad
    map.push(("daronnes", "mère")); // Mom
    map.push(("genre", "comme")); // Like (filler word)
    map.push(("grave", "totalement")); // Totally/Very
    map.push(("myth", "mensonge")); // Lie (Mytho)
    map.push(("mytho", "menteur")); // Liar

    // ==========================================
    // 4. QUEBEC SLANG (JOUAL & MODERN)
    // ==========================================
    map.push(("chum", "copain/ami")); // Boyfriend or Friend
    map.push(("blonde", "copine")); // Girlfriend
    map.push(("char", "voiture")); // Car
    map.push(("frette", "froid")); // Cold (Weather)
    map.push(("plate", "ennuyant")); // Boring
    map.push(("magané", "abimé/fatigué")); // Worn out / Tired / Damaged
    map.push(("jaser", "discuter")); // To chat
    map.push(("niaiseux", "idiot")); // Stupid/Silly
    map.push(("coche", "génial")); // Awesome (sur la coche)
    map.push(("écoeurant", "génial")); // Awesome (Context: "C'est écoeurant!" = It's sick/good)
                                       // WARNING: Can also mean "disgusting", but usually positive in slang.
    map.push(("tiguidou", "d'accord")); // Alright/Good/Agreed
    map.push(("pantoute", "pas du tout")); // Not at all
    map.push(("piasse", "dollar")); // Dollar/Money
    map.push(("bibitte", "insecte")); // Bug/Insect
    map.push(("capoter", "paniquer")); // To panic / To freak out (positive or negative)
    map.push(("lâcher un wack", "crier")); // To scream/shout
    map.push(("pogner", "attraper")); // To catch / To be popular / To understand
    map.push(("tu veux-tu", "veux-tu")); // Do you want (Quebec grammar doubling)
    map.push(("icitte", "ici")); // Here
    map.push(("asteure", "maintenant")); // Now (À cette heure)
    map.push(("tanné", "en avoir marre")); // Fed up
    map.push(("checker", "regarder")); // To look at / Check
    map.push(("canceller", "annuler")); // To cancel (Anglicism common in QC)
    map.push(("breuvage", "boisson")); // Drink (In France 'breuvage' is for animals/potions)
    map.push(("gosses", "testicules")); // Testicles (WARNING: In France this means KIDS)
                                        // Since this dictionary is likely for converting TO English,
                                        // M2M100 usually assumes France French.
                                        // Qwen needs context for this one.

    // ==========================================
    // 5. GAMING / INTERNET SPECIFIC
    // ==========================================
    map.push(("gg", "bien joué")); // Good Game
    map.push(("noob", "débutant")); // Beginner
    map.push(("lag", "ralentissement")); // Lag
    map.push(("bug", "erreur")); // Error
    map.push(("hack", "triche")); // Cheat
    map.push(("pv", "message privé")); // Private Message (MP/PV)
    map.push(("mp", "message privé")); // Private Message
    map.push(("re", "rebonjour")); // Hi again (returned)
    map.push(("ping", "latence")); // Latency
    map.push(("ban", "bannir")); // Ban
    map.push(("kick", "exclure")); // Kick
    map.push(("rush", "attaquer vite")); // Attack fast
    map.push(("camp", "rester statique")); // Camp
    map.push(("rageux", "mauvais perdant")); // Sore loser / Rager

    // ==========================================
    // 6. FRANCE: VULGAR INSULTS & SWEARS
    // ==========================================
    map.push(("merde", "zut")); // Shit (Generic)
    map.push(("putain", "mince")); // F*** / Damn (The universal French comma)
    map.push(("connard", "imbécile")); // Asshole (Male)
    map.push(("connasse", "imbécile")); // Asshole/Bitch (Female)
    map.push(("salope", "femme méchante")); // Bitch/Slut
    map.push(("pute", "prostituée")); // Whore/Bitch
    map.push(("batard", "salaud")); // Bastard
    map.push(("enculé", "salaud")); // F***er / Motherf***er (Lit: buggered)
    map.push(("nique", "coucher avec")); // F*** (e.g., "Je te nique")
    map.push(("niquer", "casser/battre")); // To f***/break/beat
    map.push(("foutre", "sperme")); // C*m (noun) / To do (verb slang)
    map.push(("chiant", "ennuyeux")); // Pain in the ass / Annoying
    map.push(("gueule", "bouche")); // Shut up (Ta gueule) / Face
    map.push(("con", "idiot")); // Stupid / C*nt (Note: 'Con' is mild in FR, often just means Idiot)
    map.push(("debile", "idiot")); // Moron

    // ==========================================
    // 7. FRANCE: ANATOMY & SEX SLANG
    // ==========================================
    map.push(("bite", "pénis")); // Dick
    map.push(("teub", "pénis")); // Dick (Verlan of bite)
    map.push(("queue", "pénis")); // Dick (Tail)
    map.push(("chatte", "vagin")); // Pussy
    map.push(("foufoune", "vagin")); // Pussy (In France. WARNING: In Quebec this usually means Butt/Funny)
    map.push(("couilles", "testicules")); // Balls
    map.push(("boule", "fesses")); // Ass (Le boule)
    map.push(("cul", "fesses")); // Ass
    map.push(("baise", "sexe")); // Sex / F***ing
    map.push(("baiser", "faire l'amour")); // To f***
    map.push(("branler", "masturber")); // To wank / To do nothing ("Rien à branler")
    map.push(("sucer", "faire une fellation")); // To suck

    // ==========================================
    // 8. FRANCE: VULGAR ACRONYMS (TEXTING)
    // ==========================================
    map.push(("fdp", "fils de pute")); // Son of a b****
    map.push(("ntm", "nique ta mère")); // F*** your mother
    map.push(("vtff", "va te faire foutre")); // Go f*** yourself
    map.push(("tg", "tais-toi")); // Shut the f*** up (Ta gueule)
    map.push(("ftg", "ferme ta gueule")); // Shut the f*** up
    map.push(("raf", "je m'en fiche")); // I don't give a f*** (Rien à foutre)
    map.push(("osef", "je m'en fiche")); // Who cares (On s'en fout)
    map.push(("balek", "je m'en fiche")); // Don't give a sh** (Bat les couilles)
    map.push(("blc", "je m'en fiche")); // Don't give a sh** (Bat les couilles)
    map.push(("oklm", "tranquille")); // Chilling (Au calme - slang)
    map.push(("klm", "tranquille")); // Chilling

    // ==========================================
    // 9. QUEBEC: "LES SACRES" (The Church Swears)
    // ==========================================
    // We map these to "putain" or "merde" so the translation model knows they are expletives.

    // The "Big Three" (Strongest)
    map.push(("tabarnak", "putain")); // F*** (Tabernacle) - The ultimate Quebec swear
    map.push(("calisse", "putain")); // Damn/F*** (Chalice)
    map.push(("crisse", "putain")); // Christ/Damn

    // Medium Intensity
    map.push(("osti", "merde")); // Shit/Damn (Host)
    map.push(("ostie", "merde")); // Shit/Damn
    map.push(("astie", "merde")); // Shit/Damn (Variation)
    map.push(("ciboire", "bordel")); // Ciborium (Damn it)
    map.push(("viarge", "merde")); // Virgin (Damn)
    map.push(("saint-crème", "mon dieu")); // Holy cream (Soft swear)
    map.push(("marde", "merde")); // Shit (Pronunciation variant)

    // "Softened" Versions (Like "Darn" or "Frick")
    map.push(("tabarouette", "zut")); // Darn (Soft Tabarnak)
    map.push(("tabarnouche", "zut")); // Darn
    map.push(("caline", "zut")); // Darn (Soft Calisse)
    map.push(("cristie", "zut")); // Darn (Soft Crisse)

    // ==========================================
    // 10. QUEBEC: SPECIFIC INSULTS
    // ==========================================
    map.push(("cave", "idiot")); // Idiot (Very common: "T'es ben cave")
    map.push(("epais", "idiot")); // Thick/Stupid ("Maudit épais")
    map.push(("sans-dessein", "idiot")); // Moron (Lit: Without design/plan)
    map.push(("colon", "ignorant")); // Hillbilly/Uncultured
    map.push(("tata", "stupide")); // Dummy
    map.push(("nounoune", "bête")); // Silly/Dumb (often used for women, or general idiot)
    map.push(("guidoune", "prostituée")); // Slut/Easy woman
    map.push(("plotte", "vagin")); // C*** / Slut (Highly offensive in QC)
    map.push(("graine", "pénis")); // Dick (Lit: Seed/Grain)
    map.push(("totons", "seins")); // Boobs
    map.push(("fif", "homosexuel")); // F*g (Homophobic slur)
    map.push(("fifi", "faible")); // Weak/Sissy

    map
}
