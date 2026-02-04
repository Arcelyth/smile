pub enum KaoMoJi {
    Smile,
    Angry,
    Sleep,
    Wink,
}

pub fn get_kaomoji(kind: KaoMoJi) -> String {
    match kind {
        KaoMoJi::Wink => "☆(>ᴗ•)",
        KaoMoJi::Angry => "(`ᴖ´)",
        KaoMoJi::Sleep=> "(ᴗ˳ᴗ)ᶻ𝗓𐰁",
        _ => ">ᴗ<"
    }.to_string()
} 
