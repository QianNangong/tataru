use rand::{
    seq::{index, SliceRandom},
    thread_rng,
};

pub enum Arcana {
    TheWorld,
    Judgement,
    TheSun,
    TheMoon,
    TheStar,
    TheTower,
    TheDevil,
    Temperance,
    Death,
    TheHangedMan,
    Justice,
    TheWheelOfFortune,
    TheHermit,
    Strength,
    TheChariot,
    TheLovers,
    TheHierophant,
    TheEmperor,
    TheHighPriestess,
    TheMagician,
    TheFool,
}

pub enum TarotCard {
    MajorArcana(Arcana),
    Wands(i16),
    Cups(i16),
    Swords(i16),
    Rentacles(i16),
}

impl TarotCard {
    pub fn get_name(&self) -> &str {
        match self {
            TarotCard::MajorArcana(arcana) => match arcana {
                Arcana::TheWorld => todo!(),
                Arcana::Judgement => todo!(),
                Arcana::TheSun => todo!(),
                Arcana::TheMoon => todo!(),
                Arcana::TheStar => todo!(),
                Arcana::TheTower => todo!(),
                Arcana::TheDevil => todo!(),
                Arcana::Temperance => todo!(),
                Arcana::Death => todo!(),
                Arcana::TheHangedMan => todo!(),
                Arcana::Justice => todo!(),
                Arcana::TheWheelOfFortune => todo!(),
                Arcana::TheHermit => todo!(),
                Arcana::Strength => todo!(),
                Arcana::TheChariot => todo!(),
                Arcana::TheLovers => todo!(),
                Arcana::TheHierophant => todo!(),
                Arcana::TheEmperor => todo!(),
                Arcana::TheHighPriestess => todo!(),
                Arcana::TheMagician => todo!(),
                Arcana::TheFool => todo!(),
            },
            TarotCard::Wands(i) => match i {
                _ => {}
            },
            TarotCard::Cups(i) => {}
            TarotCard::Swords(i) => {}
            TarotCard::Rentacles(i) => {}
        };
        ""
    }

    pub fn get_meaning(&self) {}
}

pub enum DeckType {
    Major,
    FullSet,
}

pub struct TarotDeck {
    cards: Vec<TarotCard>,
}

impl TarotDeck {
    pub fn new(deck_type: DeckType) -> Self {
        let mut cards = Vec::new();
        match deck_type {
            DeckType::Major => {}
            DeckType::FullSet => {}
        };
        cards.shuffle(&mut thread_rng());
        TarotDeck { cards: cards }
    }

    // pub fn draw(index1: i16, index2: i16, index3: i16) -> (TarotCard, TarotCard, TarotCard) {

    // }
}
