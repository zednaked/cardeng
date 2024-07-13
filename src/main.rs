use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        system::RunSystemOnce,
    },
    prelude::*,
};
use bevy_mod_picking::prelude::*;
use rand::seq::SliceRandom;
//use serde::Deserialize;
use serde::{Deserialize, Serialize};
//use serde_json::*;
use reqwest::StatusCode;

use reqwest::blocking::Client;
use serde_json::from_str;
use std::error::Error;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_seconds(0.25))
        .insert_resource(config {
            deck: Deck::default(),
            jogador: jogador::new(),
        })
        .add_event::<e_spawnar_carta>()
        .add_event::<e_atualiza_jogador>()
        .add_event::<e_atualiza_slot>()
        .add_event::<e_envia_status>()
        .add_plugins(DefaultPickingPlugins)
        .add_systems(Update, spawna_carta.run_if(on_event::<e_spawnar_carta>()))
        .add_systems(Update, atualiza_slot.run_if(on_event::<e_atualiza_slot>()))
        .add_systems(Update, atualiza_status.run_if(on_event::<e_envia_status>()))
        .add_systems(
            Update,
            atualiza_jogador.run_if(on_event::<e_atualiza_jogador>()),
        )
        .add_systems(Update, fim_dragging)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Debug, Component, Clone)]
struct LabelJogador;

fn atualiza_jogador(
    mut events: EventReader<e_atualiza_jogador>,
    mut jogador: ResMut<config>,
    mut q_texto_jogador: Query<&mut Text, With<LabelJogador>>,
) {
    for event in events.read() {
        match event.tipo {
            TipoAtualizacao::tomar_dano => {
                jogador.jogador.tomar_dano(event.valor);
            }
            TipoAtualizacao::sobe_level => {
                jogador.jogador.subir_level();
            }
        }

        for mut texto in q_texto_jogador.iter_mut() {
            texto.sections[1].value = format!(
                "Vida: {}/{}\nAtaque: {}\nDefesa: {}\nOuro: {}\nLevel: {}\nXP: {}\n",
                jogador.jogador.vida_atual,
                jogador.jogador.vida_inicial,
                jogador.jogador.ataque,
                jogador.jogador.defesa,
                jogador.jogador.ouro,
                jogador.jogador.level,
                jogador.jogador.xp
            );
        }
    }
}

pub struct jogador {
    vida_inicial: i32,
    vida_atual: i32,
    ataque: i32,
    defesa: i32,
    ouro: i32,
    level: i32,
    xp: i32,
    posicao: i32,
}

impl jogador {
    pub fn new() -> Self {
        jogador {
            vida_inicial: 10,
            vida_atual: 10,
            ataque: 1,
            defesa: 1,
            ouro: 0,
            level: 0, //significa que ele esta na entrada da dungeon
            xp: 0,
            posicao: 1, //ele começa o jogo na posicao central, podendo acessar as 3 posições na
                        //frente dele
        }
    }
    pub fn atacar(&mut self, ataque: i32) {
        self.ataque += ataque;
    }
    pub fn defender(&mut self, defesa: i32) {
        self.defesa += defesa;
    }
    pub fn curar(&mut self, cura: i32) {
        self.vida_atual += cura;
    }
    pub fn tomar_dano(&mut self, dano: i32) {
        self.vida_atual -= dano;
    }
    pub fn ganhar_ouro(&mut self, ouro: i32) {
        self.ouro += ouro;
    }
    pub fn ganhar_xp(&mut self, xp: i32) {
        self.xp += xp;
    }
    pub fn subir_level(&mut self) {
        self.level += 1;
    }
    pub fn resetar(&mut self) {
        self.vida_atual = self.vida_inicial;
        self.ataque = 1;
        self.defesa = 1;
        self.ouro = 0;
        self.level = 0;
        self.xp = 0;
        self.posicao = 1;
    }
    pub fn status(&self) {
        println!(
            "Vida: {}/{} Ataque: {} Defesa: {} Ouro: {} Level: {} XP: {}",
            self.vida_atual,
            self.vida_inicial,
            self.ataque,
            self.defesa,
            self.ouro,
            self.level,
            self.xp
        );
    }
}

enum TipoAtualizacao {
    tomar_dano,
    sobe_level,
}

#[derive(Event)]
struct e_atualiza_jogador {
    tipo: TipoAtualizacao,
    valor: i32,
}

#[derive(Resource)]
struct config {
    pub deck: Deck,
    pub jogador: jogador,
}

#[derive(Debug, Component, Clone)]
struct Status;

#[derive(Event)]
struct e_envia_status(String);

fn atualiza_status(
    mut events: EventReader<e_envia_status>,
    mut q_texto_status: Query<(&mut Text), With<(Status)>>,
) {
    //fo!("{!}", events)
    //
    for event in events.read() {
        let mut texto = q_texto_status.single_mut();
        texto.sections[0].value = format!("Status: {}", event.0);
        info!("{}", event.0);
    }
}

#[derive(Debug, Component, Clone)]
struct Dragging;

#[derive(Event)]
struct e_spawnar_carta;

/*#[derive(Debug, Component, Clone, Deserialize)]
struct Carta {
    id: i32,
    nome: String,
    descricao: String,
    ataque: i32,
    defesa: i32,
    vida: i32,
}*/

#[derive(Debug, Component, Clone)]
struct EntidadeCarta(Entity);

#[derive(Component, Clone, Serialize, Deserialize, Debug)]
struct Carta {
    id: u32,
    nome: String,
    tipo: TipoCarta,
    descricao: String,
    // Campos opcionais, dependendo do tipo de carta
    ataque: Option<i32>,
    defesa: Option<i32>,
    vida: Option<i32>,
    cura: Option<i32>,
    bonus_ataque: Option<i32>,
    bonus_defesa: Option<i32>,
    bonus_vida: Option<i32>,
    valor: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
enum TipoCarta {
    Inimigo,
    Vida,
    Equipamento,
    Artefato,
    Item,
}

impl Default for Carta {
    fn default() -> Self {
        Carta {
            id: 777,
            nome: String::from("Carta"),
            descricao: String::from("Descricao"),
            ataque: Some(0),
            defesa: Some(0),
            vida: Some(0),
            cura: Some(0),
            bonus_ataque: Some(0),
            bonus_defesa: Some(0),
            bonus_vida: Some(0),
            tipo: TipoCarta::Inimigo,
            valor: Some(0),
        }
    }
}

#[derive(Debug, Component, Clone)]
struct Deck {
    cartas: Vec<Carta>,
    level: i32,
}

impl Default for Deck {
    fn default() -> Self {
        Deck {
            cartas: Vec::new(),
            level: 1,
        }
    }
}

impl Deck {
    fn init_de_json(&mut self) {
        let json: &str = include_str!("cartas.json");
        let cartas: Vec<Carta> = serde_json::from_str(json).expect("Erro ao analisar o JSON");
        self.cartas = cartas;
        //imprime info para eu debugar
        //        for carta in self.cartas.iter() {
        //            info!("{:?}", carta.nome);
        //        }
    }

    //criar uma funcao que varre as cartas do deck, separa em tipos e atualiza o deck para que a
    //cada 4 cartas de um tipo, uma carta de outro tipo seja adiciona
    // o deck tem que ter 30 cartas
    fn monta_deck(&mut self) {
        let mut cartas_inimigo: Vec<Carta> = Vec::new();
        let mut cartas_vida: Vec<Carta> = Vec::new();
        let mut cartas_equipamento: Vec<Carta> = Vec::new();
        let mut cartas_artefato: Vec<Carta> = Vec::new();
        let mut cartas_item: Vec<Carta> = Vec::new();
        let mut cartas: Vec<Carta> = Vec::new();
        for carta in self.cartas.iter() {
            match carta.tipo {
                TipoCarta::Inimigo => cartas_inimigo.push(carta.clone()),
                TipoCarta::Vida => cartas_vida.push(carta.clone()),
                TipoCarta::Equipamento => cartas_equipamento.push(carta.clone()),
                TipoCarta::Artefato => cartas_artefato.push(carta.clone()),
                TipoCarta::Item => cartas_item.push(carta.clone()),
            }
        }
        //crie um algoritimo para montar um deck com 30 cartas com frequencias diferente dependendo
        //do tipo
        let mut rng = rand::thread_rng();
        cartas_inimigo.shuffle(&mut rng);

        //        for carta in cartas_equipamento.iter() {
        //          info!("{:?}", carta.nome);
        //    }

        //        cartas_vida.shuffle(&mut rng);
        cartas_equipamento.shuffle(&mut rng);
        cartas_artefato.shuffle(&mut rng);
        cartas_item.shuffle(&mut rng);
        let mut i = 0;
        while i < 50 {
            if cartas_inimigo.len() > 0 {
                cartas.push(cartas_inimigo.pop().unwrap_or_default());
            }

            //            if cartas_inimigo.len() > 0 {
            //              cartas.push(cartas_inimigo.pop().unwrap_or_default());
            //        }
            //      if cartas_vida.len() > 0 {
            //        cartas.push(cartas_vida.pop().unwrap_or_default());
            //  }
            //acrescenta uma chance de 1/2 de aparecer um equipamento
            if (i % 3 == 0) && cartas_equipamento.len() > 0 {
                cartas.push(cartas_equipamento.pop().unwrap_or_default());
            }
            if (i % 4 == 0) && cartas_artefato.len() > 0 {
                //acrescenta uma chance de 1/3 de aparecer um artefato
                cartas.push(cartas_artefato.pop().unwrap_or_default());
            }
            if (i % 2 == 0) && cartas_item.len() > 0 {
                //acrescenta uma chance de 1/3 de aparecer um item
                cartas.push(cartas_item.pop().unwrap_or_default());
            }

            i += 5;
        }
        cartas.shuffle(&mut rng);
        cartas.insert(
            0,
            Carta {
                id: 73373,
                nome: "Escadas".to_string(),
                descricao: "O deck está vazio!".to_string(),
                ataque: Some(0),
                defesa: Some(0),
                vida: Some(0),
                cura: Some(0),
                bonus_ataque: Some(0),
                bonus_defesa: Some(0),
                bonus_vida: Some(0),
                tipo: TipoCarta::Vida,
                valor: Some(0),
            },
        );
        cartas.insert(
            0,
            Carta {
                id: 37337,
                nome: "O Vazio".to_string(),
                descricao: "O deck está vazio!".to_string(),
                ataque: Some(0),
                defesa: Some(0),
                vida: Some(0),
                cura: Some(0),
                bonus_ataque: Some(0),
                bonus_defesa: Some(0),
                bonus_vida: Some(0),
                tipo: TipoCarta::Vida,
                valor: Some(0),
            },
        );

        cartas.insert(
            0,
            Carta {
                id: 37337,
                nome: "O Vazio".to_string(),
                descricao: "O deck está vazio!".to_string(),
                ataque: Some(0),
                defesa: Some(0),
                vida: Some(0),
                cura: Some(0),
                bonus_ataque: Some(0),
                bonus_defesa: Some(0),
                bonus_vida: Some(0),
                tipo: TipoCarta::Vida,
                valor: Some(0),
            },
        );

        cartas.insert(
            0,
            Carta {
                id: 37337,
                nome: "O Vazio".to_string(),
                descricao: "O deck está vazio!".to_string(),
                ataque: Some(0),
                defesa: Some(0),
                vida: Some(0),
                cura: Some(0),
                bonus_ataque: Some(0),
                bonus_defesa: Some(0),
                bonus_vida: Some(0),
                tipo: TipoCarta::Vida,
                valor: Some(0),
            },
        );

        cartas.insert(
            0,
            Carta {
                id: 37337,
                nome: "O Vazio".to_string(),
                descricao: "O deck está vazio!".to_string(),
                ataque: Some(0),
                defesa: Some(0),
                vida: Some(0),
                cura: Some(0),
                bonus_ataque: Some(0),
                bonus_defesa: Some(0),
                bonus_vida: Some(0),
                tipo: TipoCarta::Vida,
                valor: Some(0),
            },
        );
        //      info!("{:?}", cartas);
        self.cartas = cartas;
    }

    fn init_do_backend() -> Result<Vec<Carta>, Box<dyn Error>> {
        let client = Client::new();
        let response = client.get("http://localhost:3000/deck").send()?;

        if response.status().is_success() {
            let json_str = response.text()?;
            let cartas: Vec<Carta> = from_str(&json_str)?;
            Ok(cartas)
        } else {
            Err("Falha ao buscar cartas do backend".into())
        }
    }

    fn rm_carta(&mut self, id: u32) {
        self.cartas.retain(|x| x.id != id);
    }

    fn adc_carta(&mut self, carta: Carta) {
        self.cartas.push(carta);
    }
    fn get_primeira_carta(&mut self) -> Carta {
        self.cartas.pop().unwrap()
    }
    fn embaralhar(&mut self) -> &mut Self {
        self.cartas.shuffle(&mut rand::thread_rng());
        self
    }

    fn init(&mut self) {
        self.cartas = match Self::init_do_backend() {
            // Chama a função com Self::
            Ok(cartas) => cartas, // Em caso de sucesso, atribui as cartas
            Err(e) => {
                eprintln!("Erro ao buscar cartas do backend: {}", e);
                Vec::new() // Em caso de erro, inicializa com um vetor vazio
            }
        };
    }
}

//dependendo da localizacao do slot, no entre turnos, a carta que estiver nele vai poder ativar as
//cartas que estao no level acima
#[derive(Debug, Component, Clone)]
struct Atualizar;

#[derive(Event)]
struct e_atualiza_slot;

#[derive(Debug, Component, Clone)]
struct Slot {
    level: i32,
    carta: Carta,
    posicao: i32, //pode ser 0 para na posiçao esquerda, 1 no meio e 2 na direita do tabuleiro
    entidade_carta: Entity,
}

impl Default for Slot {
    fn default() -> Self {
        Slot {
            level: 0,
            carta: Carta::default(),
            posicao: 0,
            entidade_carta: Entity::PLACEHOLDER,
        }
    }
}

impl Slot {
    fn set_level(&mut self, level: i32) {
        self.level = level;
    }
    //esse metodo cria uma carta retirada do deck que esta disponivel nesse jogo e coloca sobre
    //esse slot
    fn adc_carta(&mut self, mut deck: Deck) {
        self.carta = deck.get_primeira_carta();
    }
}

//procura por slots que tenham componente atualizar e coloca uma carta sobre ele retirada do deck
fn atualiza_slot(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut query_slot: Query<(Entity, &mut Slot, &Transform, &Atualizar)>,
    //    mut query_deck: Query<(Entity, &mut Deck)>,
    mut query_carta: Query<(Entity, &Carta)>,
) {
    for (en_slot, mut slot, transform_slot, _) in query_slot.iter_mut() {
        //      for (_, mut deck) in query_deck.iter_mut() {
        //let carta = deck.get_primeira_carta();
        let carta = slot.carta.clone();
        if carta.nome == "Carta Vazia" {
            continue;
        }
        let cor: Color = match carta.tipo {
            TipoCarta::Inimigo => Srgba::new(1.0, 0.5, 0.5, 1.0).into(),
            TipoCarta::Vida => Srgba::new(0.5, 1.0, 0.5, 1.0).into(),
            TipoCarta::Equipamento => Srgba::new(0.5, 0.5, 1.0, 1.0).into(),
            TipoCarta::Artefato => Srgba::new(1.0, 1.0, 0.5, 1.0).into(),
            TipoCarta::Item => Srgba::new(0.5, 0.5, 0.5, 1.0).into(),
        };

        let carta_img: Handle<Image> = asset_server.load("carta.png");
        let carta_id = commands
            .spawn((
                Ancora {
                    x: transform_slot.translation.x,
                    y: transform_slot.translation.y,
                },
                PickableBundle::default(),
                SpriteBundle {
                    sprite: Sprite {
                        color: cor,
                        ..Default::default()
                    },

                    texture: carta_img.clone(),
                    transform: Transform::from_xyz(
                        transform_slot.translation.x,
                        transform_slot.translation.y,
                        1.,
                    )
                    .with_scale(Vec3::splat(0.5)),
                    ..Default::default()
                },
            ))
            .id();
        //spawnar os dados da carta
        commands.entity(carta_id).insert(carta.clone());
        //spawnar os labels da carta
        let texto_carta_id = commands
            .spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: carta.nome.clone(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 39.0,
                            color: Color::BLACK,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 0., 1.),
                ..Default::default()
            })
            .id();

        if carta.tipo == TipoCarta::Inimigo {
            let texto_carta_ataque = commands
                .spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("ata:{}", carta.ataque.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 39.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(20., -100., 1.),
                    ..Default::default()
                })
                .id();
            let texto_carta_defesa = commands
                .spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("def: {}", carta.defesa.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 39.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(20., 100., 1.),
                    ..Default::default()
                })
                .id();
            commands.entity(carta_id).push_children(&[texto_carta_id]);
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_ataque]);
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_defesa]);
        }
        if carta.tipo == TipoCarta::Artefato {
            let texto_carta_ataque = commands
                .spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("ata:{}", carta.ataque.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 39.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(20., -100., 1.),
                    ..Default::default()
                })
                .id();
            let texto_carta_defesa = commands
                .spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("def: {}", carta.defesa.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 39.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(20., 100., 1.),
                    ..Default::default()
                })
                .id();
            commands.entity(carta_id).push_children(&[texto_carta_id]);
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_ataque]);
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_defesa]);
        }
        if carta.tipo == TipoCarta::Equipamento {
            if carta.bonus_ataque.unwrap_or_default() > 0 {
                let texto_carta_ataque = commands
                    .spawn(Text2dBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: format!("ata + {}", carta.bonus_ataque.unwrap_or_default()),
                                style: TextStyle {
                                    //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 39.0,
                                    color: Color::BLACK,
                                    ..Default::default()
                                },
                            }],
                            ..Default::default()
                        },

                        transform: Transform::from_xyz(20., -100., 1.),
                        ..Default::default()
                    })
                    .id();
                commands.entity(carta_id).push_children(&[texto_carta_id]);
                commands
                    .entity(carta_id)
                    .push_children(&[texto_carta_ataque]);
            }
            if carta.bonus_defesa.unwrap_or_default() > 0 {
                let texto_carta_defesa = commands
                    .spawn(Text2dBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: format!("def + {}", carta.bonus_defesa.unwrap_or_default()),
                                style: TextStyle {
                                    //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 39.0,
                                    color: Color::BLACK,
                                    ..Default::default()
                                },
                            }],
                            ..Default::default()
                        },

                        transform: Transform::from_xyz(20., -80., 1.),
                        ..Default::default()
                    })
                    .id();
                commands.entity(carta_id).push_children(&[texto_carta_id]);
                commands
                    .entity(carta_id)
                    .push_children(&[texto_carta_defesa]);
            }
        }

        if carta.tipo == TipoCarta::Item {
            let texto_carta_valor = commands
                .spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("$${}$$", carta.valor.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 39.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(0., -100., 1.),
                    ..Default::default()
                })
                .id();
            commands.entity(carta_id).push_children(&[texto_carta_id]);
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_valor]);
        }

        slot.entidade_carta = carta_id;
        commands.entity(en_slot).remove::<Atualizar>();
    }
    //}
}

#[derive(Debug, Component, Clone)]
struct ProcessaFimDragging;

#[derive(Debug, Component, Clone)]
struct Ancora {
    x: f32,
    y: f32,
}

fn fim_dragging(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &ProcessaFimDragging,
            &mut Ancora,
            //            &Slot,
        ),
        (Without<Slot>, With<Carta>),
    >,

    mut q_camera: Query<(&mut Transform, &Camera2d), (Without<Carta>, With<Camera2d>)>,
    mut asset_server: Res<AssetServer>,
    mut jogador: ResMut<config>,
    mut query_slots: Query<(Entity, &mut Slot, &Transform), (With<Slot>, Without<Camera2d>)>,
    mut query_cartas: Query<(Entity, &Carta), Without<ProcessaFimDragging>>,
    mut ew_envia_status: EventWriter<e_envia_status>,
    mut ew_atualiza_jogador: EventWriter<e_atualiza_jogador>,
    mut ew_atualiza_slot: EventWriter<e_atualiza_slot>,
) {
    //o jogo deve verificar o level do jogador e só deixar ele interagir com o proximo level, caso
    //contrario manda uma msg via status dizendo que ele não tem level suficiente

    for (entity, mut transform_carta, _, mut ancora_carta) in query.iter_mut() {
        commands.entity(entity).remove::<ProcessaFimDragging>();
        commands.entity(entity).remove::<Dragging>();
        for (entity_slot, mut slot, transform_slot) in query_slots.iter_mut() {
            if (transform_carta.translation.x - transform_slot.translation.x).abs() < 50.
                && (transform_carta.translation.y - transform_slot.translation.y).abs() < 50.
            {
                if slot.level > jogador.jogador.level {
                    ew_envia_status
                        .send(e_envia_status("Voce nao tem level suficiente".to_string()));

                    transform_carta.translation.x = ancora_carta.x;
                    transform_carta.translation.y = ancora_carta.y;
                    return;
                }

                if slot.level < jogador.jogador.level {
                    ew_envia_status.send(e_envia_status("Nao existe retorno".to_string()));

                    transform_carta.translation.x = ancora_carta.x;
                    transform_carta.translation.y = ancora_carta.y;
                    return;
                }
                if jogador.jogador.posicao == 0 && slot.posicao == 2 {
                    ew_envia_status.send(e_envia_status(
                        "Voce nao pode interagir com esse slot".to_string(),
                    ));
                    transform_carta.translation.x = ancora_carta.x;
                    transform_carta.translation.y = ancora_carta.y;
                    return;
                }
                if jogador.jogador.posicao == 2 && slot.posicao == 0 {
                    ew_envia_status.send(e_envia_status(
                        "Voce nao pode interagir com esse slot".to_string(),
                    ));
                    transform_carta.translation.x = ancora_carta.x;
                    transform_carta.translation.y = ancora_carta.y;
                    return;
                }

                //desespawna a carta de baixo
                for (entity_carta, carta) in query_cartas.iter() {
                    if carta.id == slot.carta.id {
                        //     commands.entity(entity_carta).despawn_recursive();
                        if entity_carta != entity {
                            if slot.entidade_carta == entity_carta {
                                if carta.nome == "Escadas" {
                                    commands.entity(entity_carta).despawn_recursive();
                                    ew_envia_status.send(e_envia_status(
                                        "Voce chegou até as escadas para baixo...".to_string(),
                                    ));
                                    return;
                                }
                                if carta.nome == "O Vazio" {
                                    commands.entity(entity_carta).despawn_recursive();
                                    ew_envia_status.send(e_envia_status(
                                        "Voce caiu no vazio... GAME OVER".to_string(),
                                    ));
                                    return;
                                }

                                commands.entity(slot.entidade_carta).despawn_recursive();
                                slot.entidade_carta = Entity::PLACEHOLDER;
                            }
                        }
                    }
                }

                //se chegou aqui significa que a posição é valida, agora o jogo deve criar mais 3 cartas no final
                //das ultimas e mover a camera o suficiente para manter as novas cartas visiveis
                //cria-se primeiro os slots e marca eles para Atualizar, depois cria-se as cartas
                let mut deck = jogador.deck.clone();

                let slot_img = asset_server.load("cemiterio.png");
                deck.level += 1;
                //muda a posicao do jogador no eixo horizontal
                jogador.jogador.posicao = slot.posicao;

                match slot.carta.tipo {
                    TipoCarta::Inimigo => {
                        ew_envia_status
                            .send(e_envia_status("Voce encontrou um inimigo".to_string()));
                        jogador.jogador.xp += slot.carta.ataque.unwrap_or_default();
                        ew_atualiza_jogador.send(e_atualiza_jogador {
                            tipo: TipoAtualizacao::tomar_dano,
                            valor: 1,
                        });
                    }
                    TipoCarta::Vida => {
                        ew_envia_status.send(e_envia_status("Voce encontrou um item".to_string()));
                        jogador.jogador.vida_atual += 1;
                    }
                    TipoCarta::Equipamento => {
                        ew_envia_status
                            .send(e_envia_status("Voce encontrou um equipamento".to_string()));
                        //verifica o bonus do equipamento e adiciona ao jogador
                        if let Some(bonus_ataque) = slot.carta.bonus_ataque {
                            jogador.jogador.ataque += bonus_ataque;
                        }
                        if let Some(bonus_defesa) = slot.carta.bonus_defesa {
                            jogador.jogador.defesa += bonus_defesa;
                        }
                        if let Some(bonus_vida) = slot.carta.bonus_vida {
                            jogador.jogador.vida_atual += bonus_vida;
                        }
                    }
                    TipoCarta::Artefato => {
                        ew_envia_status
                            .send(e_envia_status("Voce encontrou um artefato".to_string()));
                        //artefatos acrescentam um bonus ao jogador

                        if let Some(bonus_ataque) = slot.carta.bonus_ataque {
                            jogador.jogador.ataque += bonus_ataque;
                        }
                        if let Some(bonus_defesa) = slot.carta.bonus_defesa {
                            jogador.jogador.defesa += bonus_defesa;
                        }
                        if let Some(bonus_vida) = slot.carta.bonus_vida {
                            jogador.jogador.vida_atual += bonus_vida;
                        }
                    }
                    TipoCarta::Item => {
                        ew_envia_status.send(e_envia_status("Voce encontrou um item".to_string()));
                        // adiciona dinheiro ao jogador
                        if let Some(valor) = slot.carta.valor {
                            jogador.jogador.ouro += valor;
                        }
                    }
                }

                //cria mais 3 slots
                for i in 1..4 {
                    slot.carta = deck.cartas.pop().unwrap_or_else(|| Carta {
                        id: 0,
                        nome: "Carta Vazia".to_string(),
                        descricao: "O deck está vazio!".to_string(),
                        ataque: Some(0),
                        defesa: Some(0),
                        vida: Some(0),
                        cura: Some(0),
                        bonus_ataque: Some(0),
                        bonus_defesa: Some(0),
                        bonus_vida: Some(0),

                        tipo: TipoCarta::Vida,
                        valor: Some(0),
                    });
                    slot.set_level(deck.level);
                    slot.posicao = i - 1; //esta começando do 1
                    commands.spawn((
                        PickableBundle::default(),
                        slot.clone(),
                        Atualizar,
                        SpriteBundle {
                            sprite: Sprite {
                                color: Srgba::new(0.5, 0.5, 0.5, 0.5).into(),
                                ..Default::default()
                            },
                            //  sprite: Sprite::new(Vec2::new(100.0, 100.0)),
                            texture: slot_img.clone(),
                            transform: Transform::from_xyz(
                                100. * i as f32,
                                (130. * deck.level as f32) - 70.,
                                0.,
                            )
                            .with_scale(Vec3::splat(0.5)),

                            ..Default::default()
                        },
                    ));
                }

                ancora_carta.x = transform_slot.translation.x;
                ancora_carta.y = transform_slot.translation.y;
                jogador.deck = deck.clone();
                commands.entity(entity_slot).insert(slot.clone());
            }
        }
        //        jogador.jogador.level += 1;
        //manda a carta pra ancora dela
        transform_carta.translation.x = ancora_carta.x;
        transform_carta.translation.y = ancora_carta.y;
        //move a camera o suficiente para caber as novas cartas spawnadas
        for (mut transform, _) in q_camera.iter_mut() {
            transform.translation.y += 100.;
        }

        ew_atualiza_slot.send(e_atualiza_slot);
        ew_atualiza_jogador.send(e_atualiza_jogador {
            tipo: TipoAtualizacao::sobe_level,
            valor: 1,
        });
    }
}

fn setup(
    mut commands: Commands,
    mut asset_server: Res<AssetServer>,
    mut res_deck: ResMut<config>,
    mut ew_spawna_carta: EventWriter<e_spawnar_carta>,
    mut ew_atualiza_slot: EventWriter<e_atualiza_slot>,
    mut ew_envia_status: EventWriter<e_envia_status>,
) {
    let deck_img = asset_server.load("deck.png");
    let slot_img = asset_server.load("cemiterio.png");

    commands.spawn(Camera2dBundle::default());
    let mut deck = Deck::default();
    deck.init_de_json();
    deck.monta_deck();

    commands.spawn((
        PickableBundle::default(),
        deck.clone(),
        On::<Pointer<Down>>::run(spawna_carta),
        SpriteBundle {
            texture: deck_img.clone(),
            transform: Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(0.5)),

            ..Default::default()
        },
    ));
    let mut slot = Slot::default();
    for ii in 0..3 {
        for i in 1..4 {
            // let mut slot = Slot::default();
            //slot.adc_carta(deck.clone());

            // slot.carta = deck.cartas.pop().unwrap();
            slot.carta = deck.cartas.pop().unwrap_or_else(|| Carta {
                id: 0,
                nome: "Carta Vazia".to_string(),
                descricao: "O deck está vazio!".to_string(),
                ataque: Some(0),
                defesa: Some(0),
                vida: Some(0),
                cura: Some(0),
                bonus_ataque: Some(0),
                bonus_defesa: Some(0),
                bonus_vida: Some(0),

                tipo: TipoCarta::Vida,
                valor: Some(0),
            });
            slot.set_level(ii);
            slot.posicao = i - 1; //esta começando do 1
            commands.spawn((
                PickableBundle::default(),
                slot.clone(),
                Atualizar,
                SpriteBundle {
                    sprite: Sprite {
                        color: Srgba::new(0.5, 0.5, 0.5, 0.5).into(),
                        ..Default::default()
                    },
                    //  sprite: Sprite::new(Vec2::new(100.0, 100.0)),
                    texture: slot_img.clone(),
                    transform: Transform::from_xyz(100. * i as f32, (130. * ii as f32) - 70., 0.)
                        .with_scale(Vec3::splat(0.5)),

                    ..Default::default()
                },
            ));
        }
    }

    deck.level = 2;

    commands.spawn((
        PickableBundle::default(),
        Slot::default(),
        SpriteBundle {
            //  sprite: Sprite::new(Vec2::new(100.0, 100.0)),
            texture: slot_img.clone(),
            transform: Transform::from_xyz(100. * 2 as f32, -195., 0.).with_scale(Vec3::splat(0.5)),

            ..Default::default()
        },
    ));

    //    ew_spawna_carta.send(e_spawnar_carta);
    ew_atualiza_slot.send(e_atualiza_slot);

    let carta_img: Handle<Image> = asset_server.load("carta.png");
    let carta_id = commands
        .spawn((
            Carta {
                id: 666,
                nome: "Heroi".to_string(),
                descricao: "boladao".to_string(),
                ataque: Some(1),
                defesa: Some(2),
                vida: Some(10),
                cura: Some(0),
                bonus_ataque: Some(0),
                bonus_defesa: Some(0),
                bonus_vida: Some(0),
                tipo: TipoCarta::Inimigo,
                valor: Some(0),
            },
            Ancora { x: 200., y: -195. },
            PickableBundle::default(),
            On::<Pointer<DragStart>>::target_insert(Dragging),
            On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                transform.translation.x += drag.delta.x;
                transform.translation.y -= drag.delta.y;
            }),
            On::<Pointer<DragEnd>>::target_insert(ProcessaFimDragging),
            SpriteBundle {
                texture: carta_img.clone(),
                transform: Transform::from_xyz(200., -195., 2.).with_scale(Vec3::splat(0.5)),
                ..Default::default()
            },
        ))
        .id();

    //spawnar os labels da carta
    let texto_carta_id = commands
        .spawn((
            LabelJogador,
            Text2dBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: "Heroi\n".to_string(),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 38.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        },
                        TextSection {
                            value: "".to_string(),
                            style: TextStyle {
                                //            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 25.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        },
                    ],
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 23., 1.),
                ..Default::default()
            },
        ))
        .id();
    commands.entity(carta_id).push_children(&[texto_carta_id]);

    //    let mut world = World::new();
    //    world.run_system_once(spawna_carta);
    //cria label pro status com o componente

    let mut status = commands.spawn((
        TextBundle::from("Status: "),
        Status,
        //    Transform::from_xyz(300., 0., 1.),
    ));
    status.insert(Transform::from_xyz(300., 0., 1.));

    res_deck.deck = deck.clone();
    ew_envia_status.send(e_envia_status("Voce entrou na dungeon".to_string()));
}

fn spawna_carta(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    //faz uma query para puxar as informaçoes do componente deck e spawnar a carta usando as cartas do
    //deck, usando get get_primeira_carta
    mut query_deck: Query<(Entity, &mut Deck)>,
) {
    //info com query query_deck

    for (_, mut deck) in query_deck.iter_mut() {
        // if deck.cartas.len() > 0 {
        let carta = deck.get_primeira_carta();
        let carta_img: Handle<Image> = asset_server.load("carta.png");
        let carta_id = commands
            .spawn((
                Ancora { x: -100., y: 0. },
                PickableBundle::default(),
                On::<Pointer<DragStart>>::target_insert(Dragging),
                On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                    transform.translation.x += drag.delta.x;
                    transform.translation.y -= drag.delta.y;
                }),
                On::<Pointer<DragEnd>>::target_insert(ProcessaFimDragging),
                SpriteBundle {
                    texture: carta_img.clone(),
                    transform: Transform::from_xyz(-100., 0., 1.).with_scale(Vec3::splat(0.5)),
                    ..Default::default()
                },
            ))
            .id();
        //spawnar os dados da carta
        commands.entity(carta_id).insert(carta.clone());
        //spawnar os labels da carta
        let texto_carta_id = commands
            .spawn(Text2dBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: carta.nome.clone(),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 20.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        },
                        TextSection {
                            value: match carta.ataque {
                                Some(ataque) => format!("\n{:?}", Some(carta.ataque)),
                                None => "".to_string(),
                            },
                            style: TextStyle {
                                //            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 15.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        },
                    ],
                    ..Default::default()
                },
                transform: Transform::from_xyz(0., 0., 1.),
                ..Default::default()
            })
            .id();
        commands.entity(carta_id).push_children(&[texto_carta_id]);
        // }
    }
}
