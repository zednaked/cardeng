#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_parens)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(unreachable_patterns)]
#![allow(for_loops_over_fallibles)]
#![allow(unused_imports)]

use bevy::{
    ecs::{
        query::{QueryData, QueryFilter},
        system::RunSystemOnce,
        world,
    },
    prelude::*,
};
use bevy_mod_picking::prelude::*;
use lens::TransformPositionLens;
use rand::seq::SliceRandom;
//use serde::Deserialize;
use serde::{Deserialize, Serialize};
//use serde_json::*;
use reqwest::StatusCode;

use reqwest::blocking::Client;
use serde_json::from_str;
use std::error::Error;

use bevy_tweening::*;
use std::time::Duration;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TweeningPlugin)
        .insert_resource(Time::<Fixed>::from_seconds(0.25))
        .insert_resource(config {
            deck: Deck::default(),
            jogador: jogador::new(),
            efeitos_inventario: EfeitosInventario::default(),
        })
        //      .add_event::<e_spawnar_carta>()
        .add_event::<e_monta_jogo>()
        .add_event::<e_resetar_jogo>()
        .add_event::<e_atualiza_jogador>()
        .add_event::<e_atualiza_slot>()
        .add_event::<e_envia_status>()
        .add_plugins(DefaultPickingPlugins)
        //        .add_systems(Update, montar_jogo.run_if(on_event::<e_monta_jogo>()))
        .add_systems(
            Update,
            (resetar_jogo, montar_jogo, atualiza_slot)
                .chain()
                .run_if(on_event::<e_resetar_jogo>()),
        )
        //        .add_systems(Update, spawna_carta.run_if(on_event::<e_spawnar_carta>()))
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
struct EfeitoInventario {
    nome: String,
    buff: i32,
    debuff: i32,
}

struct EfeitosInventario {
    efeitos: Vec<EfeitoInventario>,
}
impl Default for EfeitosInventario {
    fn default() -> Self {
        EfeitosInventario {
            efeitos: vec![EfeitoInventario {
                nome: "Espada".to_string(),
                buff: 1,
                debuff: 0,
            }],
        }
    }
}

fn atualiza_jogador(
    mut events: EventReader<e_atualiza_jogador>,
    mut jogador: ResMut<config>,
    mut q_efeitos_inventario: Query<(&mut Transform, &mut Text), With<UIEfeitosInventario>>,
    mut q_texto_jogador: Query<
        (&mut Text, &UiCartaJogador),
        (With<LabelJogador>, Without<UIEfeitosInventario>),
    >,
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
        for (mut transform, mut texto) in q_efeitos_inventario.iter_mut() {
            let mut efeitos_inventario = "Efeitos do Inventario :\n".to_string();

            for efeito in jogador.efeitos_inventario.efeitos.iter() {
                efeitos_inventario =
                    format!("{}\n{}:{}", efeitos_inventario, efeito.nome, efeito.buff);
            }
            texto.sections[0].value = efeitos_inventario;

            //   texto.sections[0].value = format!("{}", event.valor);
            //
        }

        for (mut texto, ui_carta_jogador) in q_texto_jogador.iter_mut() {
            match ui_carta_jogador.tipo {
                TipoUiCartaJogador::Ataque => {
                    texto.sections[0].value = format!("{}", jogador.jogador.ataque);
                }
                TipoUiCartaJogador::Defesa => {
                    texto.sections[0].value = format!("{}", jogador.jogador.defesa);
                }
                TipoUiCartaJogador::Ouro => {
                    texto.sections[0].value = format!("{}", jogador.jogador.ouro);
                }
                TipoUiCartaJogador::Level => {
                    texto.sections[0].value = format!("{}", jogador.jogador.level);
                }
                TipoUiCartaJogador::Xp => {
                    texto.sections[0].value = format!("{}", jogador.jogador.xp);
                }
                TipoUiCartaJogador::Vida => {
                    texto.sections[0].value = format!("{}", jogador.jogador.vida_atual);
                }
            }

            //texto.sections[0].value = format!("{}", jogador.jogador.vida_atual);
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
    pub efeitos_inventario: EfeitosInventario,
}

#[derive(Debug, Component, Clone)]
struct Status;

#[derive(Event)]
struct e_envia_status(String);

fn atualiza_status(
    mut events: EventReader<e_envia_status>,
    mut q_texto_status: Query<(&mut Text, &Status), With<Status>>,
) {
    // info!("{}", events.0);
    let mut texto_montado: String = "Status: ".to_string();
    for event in events.read() {
        texto_montado = format!("{}\n{}", texto_montado, event.0);
        //        info!("{}", event.0);
    }
    if q_texto_status.iter().count() == 0 {
        return;
    }
    if q_texto_status.iter().count() > 1 {
        return;
    }
    //q_texto_status.single();

    for (mut texto, _) in q_texto_status.iter_mut() {
        texto.sections[0].value = texto_montado.clone();
        //     info!("{}", texto.sections[0].value);
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
    Vazio,
    Escadas,
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

    fn monta_deck(&mut self) {
        let mut cartas_inimigo: Vec<Carta> = Vec::new();
        let mut cartas_vida: Vec<Carta> = Vec::new();
        let mut cartas_equipamento: Vec<Carta> = Vec::new();
        let mut cartas_artefato: Vec<Carta> = Vec::new();
        let mut cartas_item: Vec<Carta> = Vec::new();
        let mut cartas: Vec<Carta> = Vec::new();
        for carta in self.cartas.iter() {
            match carta.tipo {
                TipoCarta::Vazio => cartas.push(carta.clone()),
                TipoCarta::Escadas => cartas.push(carta.clone()),

                TipoCarta::Inimigo => cartas_inimigo.push(carta.clone()),
                TipoCarta::Vida => cartas_vida.push(carta.clone()),
                TipoCarta::Equipamento => cartas_equipamento.push(carta.clone()),
                TipoCarta::Artefato => cartas_artefato.push(carta.clone()),
                TipoCarta::Item => cartas_item.push(carta.clone()),
            }
        }

        let mut rng = rand::thread_rng();
        cartas_inimigo.shuffle(&mut rng);

        cartas_equipamento.shuffle(&mut rng);
        cartas_artefato.shuffle(&mut rng);
        cartas_item.shuffle(&mut rng);
        let mut i = 0;
        while i < 50 {
            if cartas_inimigo.len() > 0 {
                cartas.push(cartas_inimigo.pop().unwrap_or_default());
            }

            if (i % 3 == 0) && cartas_equipamento.len() > 0 {
                cartas.push(cartas_equipamento.pop().unwrap_or_default());
            }
            if (i % 4 == 0) && cartas_artefato.len() > 0 {
                cartas.push(cartas_artefato.pop().unwrap_or_default());
            }
            if (i % 2 == 0) && cartas_item.len() > 0 {
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
                tipo: TipoCarta::Escadas,
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
                tipo: TipoCarta::Vazio,
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
                tipo: TipoCarta::Vazio,
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
                tipo: TipoCarta::Vazio,
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
                tipo: TipoCarta::Vazio,
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
    mut config: ResMut<config>,
    mut query_slot: Query<(Entity, &mut Slot, &Transform, &Atualizar)>,
    //    mut query_deck: Query<(Entity, &mut Deck)>,
    mut query_carta: Query<(Entity, &Carta)>,
) {
    for (en_slot, mut slot, transform_slot, _) in query_slot.iter_mut() {
        //      for (_, mut deck) in query_deck.iter_mut() {
        //let carta = deck.get_primeira_carta();
        let index_carta = config.deck.level;

        let carta = slot.carta.clone();
        let tween_carta_deck_para_slot = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(300),
            TransformPositionLens {
                start: Vec3::new(-200., 0., 0.),
                end: Vec3::new(
                    transform_slot.translation.x,
                    transform_slot.translation.y,
                    index_carta as f32,
                ),
            },
        );
        let cor: Color = match carta.tipo {
            TipoCarta::Vazio => Srgba::new(0.1, 0.2, 0.4, 1.0).into(),
            TipoCarta::Escadas => Srgba::new(0.1, 0.1, 0.2, 1.0).into(),
            TipoCarta::Inimigo => Srgba::new(1.0, 0.5, 0.5, 1.0).into(),
            TipoCarta::Vida => Srgba::new(0.5, 1.0, 0.5, 1.0).into(),
            TipoCarta::Equipamento => Srgba::new(0.5, 0.5, 1.0, 1.0).into(),
            TipoCarta::Artefato => Srgba::new(1.0, 0.3, 1.0, 1.0).into(),
            TipoCarta::Item => Srgba::new(0.8, 0.3, 0.5, 1.0).into(),
        };

        // let carta_img: Handle<Image> = asset_server.load("carta.png");

        let carta_img: Handle<Image> = match carta.tipo {
            TipoCarta::Vazio => asset_server.load("carta-roxa.png"),
            TipoCarta::Escadas => asset_server.load("carta-roxa.png"),
            TipoCarta::Inimigo => asset_server.load("carta-vermelha.png"),
            TipoCarta::Vida => asset_server.load("carta-verde.png"),
            TipoCarta::Equipamento => asset_server.load("carta-verde.png"),
            TipoCarta::Artefato => asset_server.load("carta-amarela.png"),
            TipoCarta::Item => asset_server.load("carta-amarela.png"),
        };

        let carta_id = commands
            .spawn((
                Animator::new(tween_carta_deck_para_slot),
                Ancora {
                    x: transform_slot.translation.x,
                    y: transform_slot.translation.y,
                },
                PickableBundle::default(),
                SpriteBundle {
                    sprite: Sprite {
                        //                        color: cor,
                        ..Default::default()
                    },

                    texture: carta_img.clone(),
                    transform: Transform::from_xyz(
                        transform_slot.translation.x,
                        transform_slot.translation.y,
                        1.,
                    )
                    .with_scale(Vec3::splat(1.0)),
                    ..Default::default()
                },
            ))
            .id();
        //spawnar os dados da carta
        commands.entity(carta_id).insert(carta.clone());
        //spawnar os labels da carta
        //        info!("{}", carta.nome);
        let texto_carta_id = commands
            .spawn(Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: carta.nome.clone(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 33.0,
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
                            value: format!("{}", carta.ataque.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 32.0,
                                color: Color::WHITE,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(-50., 85., 1.),
                    ..Default::default()
                })
                .id();
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_ataque]);
        }
        if carta.tipo == TipoCarta::Artefato {
            let texto_carta_efeito = commands
                .spawn(Text2dBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("{}", carta.ataque.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 39.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(-50., 85., 1.),
                    ..Default::default()
                })
                .id();
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_efeito]);
        }
        if carta.tipo == TipoCarta::Equipamento {
            if carta.bonus_ataque.unwrap_or_default() > 0 {
                let texto_carta_ataque = commands
                    .spawn(Text2dBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: format!("{}", carta.bonus_ataque.unwrap_or_default()),
                                style: TextStyle {
                                    //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 30.0,
                                    color: Color::BLACK,
                                    ..Default::default()
                                },
                            }],
                            ..Default::default()
                        },

                        transform: Transform::from_xyz(-50., 85., 1.),
                        ..Default::default()
                    })
                    .id();
                commands
                    .entity(carta_id)
                    .push_children(&[texto_carta_ataque]);
            }
            if carta.bonus_defesa.unwrap_or_default() > 0 {
                let texto_carta_defesa = commands
                    .spawn(Text2dBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: format!("{}", carta.bonus_defesa.unwrap_or_default()),
                                style: TextStyle {
                                    //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                    font_size: 30.0,
                                    color: Color::BLACK,
                                    ..Default::default()
                                },
                            }],
                            ..Default::default()
                        },

                        transform: Transform::from_xyz(-50., 85., 1.),
                        ..Default::default()
                    })
                    .id();
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
                            value: format!("{}", carta.valor.unwrap_or_default()),
                            style: TextStyle {
                                //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 33.0,
                                color: Color::BLACK,
                                ..Default::default()
                            },
                        }],
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(-50., 85., 1.),
                    ..Default::default()
                })
                .id();
            commands
                .entity(carta_id)
                .push_children(&[texto_carta_valor]);
        }

        commands.entity(carta_id).push_children(&[texto_carta_id]);
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
    mut q_texto_status: Query<Entity, With<Status>>,
    mut q_texto_inventario: Query<Entity, With<UIEfeitosInventario>>,
    mut q_camera: Query<
        (
            Entity,
            &mut Transform,
            &mut Camera2d,
            &mut OrthographicProjection,
        ),
        (Without<Carta>, With<Camera2d>),
    >,
    mut asset_server: Res<AssetServer>,
    mut jogador: ResMut<config>,
    mut query_slots: Query<(Entity, &mut Slot, &Transform), (With<Slot>, Without<Camera2d>)>,
    mut query_cartas: Query<(Entity, &Carta), Without<ProcessaFimDragging>>,
    mut ew_envia_status: EventWriter<e_envia_status>,
    mut ew_atualiza_jogador: EventWriter<e_atualiza_jogador>,
    mut ew_atualiza_slot: EventWriter<e_atualiza_slot>,
    mut ew_resetar_jogo: EventWriter<e_resetar_jogo>,
) {
    //o jogo deve verificar o level do jogador e só deixar ele interagir com o proximo level, caso
    //contrario manda uma msg via status dizendo que ele não tem level suficiente

    for (entity, mut transform_carta, _, mut ancora_carta) in query.iter_mut() {
        commands.entity(entity).remove::<ProcessaFimDragging>();
        commands.entity(entity).remove::<Dragging>();
        for (entity_slot, mut slot, transform_slot) in query_slots.iter_mut() {
            if (transform_carta.translation.x - transform_slot.translation.x).abs() < 150.
                && (transform_carta.translation.y - transform_slot.translation.y).abs() < 150.
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
                                    //                                    commands.entity(entity_carta).despawn_recursive();
                                    ew_envia_status.send(e_envia_status(
                                        "Voce chegou até as escadas para baixo...".to_string(),
                                    ));
                                    ew_resetar_jogo.send(e_resetar_jogo);

                                    return;
                                }
                                if carta.nome == "O Vazio" {
                                    //                                  commands.entity(entity_carta).despawn_recursive();
                                    ew_envia_status.send(e_envia_status(
                                        "Voce caiu no vazio... GAME OVER".to_string(),
                                    ));
                                    ew_resetar_jogo.send(e_resetar_jogo);
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
                    TipoCarta::Escadas => {
                        ew_envia_status
                            .send(e_envia_status("Voce encontrou as escadas".to_string()));
                        jogador.jogador.level += 1;
                        jogador.jogador.xp += 1;
                        //                  jogador.jogador.ouro += 1;
                    }
                    TipoCarta::Vazio => {
                        ew_envia_status.send(e_envia_status("Voce caiu no vazio".to_string()));
                        //                jogador.jogador.vida_atual -= 1;
                    }
                    TipoCarta::Inimigo => {
                        ew_envia_status
                            .send(e_envia_status("Voce encontrou um inimigo".to_string()));
                        //verifica Defesa do player + bonus de defesa, - ataque do inimigo = dano
                        let defesa =
                            jogador.jogador.defesa + slot.carta.bonus_defesa.unwrap_or_default();
                        let dano = defesa - slot.carta.ataque.unwrap_or_default();
                        jogador.jogador.xp += slot.carta.valor.unwrap_or_default();
                        jogador.jogador.ouro += slot.carta.valor.unwrap_or_default();
                        jogador.jogador.vida_atual -= dano.abs();
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
                        //verifica se existe um equipamento com esse nome e substitui por o novo
                        let mut existe = false;
                        for efeito in jogador.efeitos_inventario.efeitos.iter_mut() {
                            if efeito.nome == slot.carta.nome {
                                efeito.buff = slot.carta.bonus_ataque.unwrap_or_default();
                                efeito.debuff = slot.carta.bonus_defesa.unwrap_or_default();
                                existe = true;
                            }
                        }
                        if !existe {
                            jogador.efeitos_inventario.efeitos.push(EfeitoInventario {
                                nome: slot.carta.nome.clone(),
                                buff: slot.carta.bonus_ataque.unwrap_or_default(),
                                debuff: slot.carta.bonus_defesa.unwrap_or_default(),
                                // descricao: slot.carta.descricao.clone(),
                            });
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

                        tipo: TipoCarta::Vazio,
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
                                (190. * i as f32) - 300.,
                                (255. * deck.level as f32) + 70.,
                                0.,
                            )
                            .with_scale(Vec3::splat(1.0)),

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
        let tween_carta_retorna_ancora = Tween::new(
            EaseFunction::QuadraticInOut,
            Duration::from_millis(400),
            TransformPositionLens {
                start: transform_carta.translation,
                end: Vec3::new(ancora_carta.x, ancora_carta.y, 8.),
            },
        );
        commands
            .entity(entity)
            .insert(Animator::new(tween_carta_retorna_ancora));
        // transform_carta.translation.x = ancora_carta.x;
        // transform_carta.translation.y = ancora_carta.y;
        //move a camera o suficiente para caber as novas cartas spawnadas
        for (entidade_camera, mut transform, _, mut op) in q_camera.iter_mut() {
            let tween_camera_sobe = Tween::new(
                EaseFunction::QuadraticInOut,
                Duration::from_millis(500),
                TransformPositionLens {
                    start: transform.translation,
                    end: Vec3::new(transform.translation.x, transform.translation.y + 250., 0.),
                },
            );

            //transform.translation.y += 250.;
            commands
                .entity(entidade_camera)
                .insert(Animator::new(tween_camera_sobe));

            //    op.scale = 1.5;
        }
        for entidade_texto_status in q_texto_status.iter() {
            //            commands
            //              .entity(entidade_texto_status)
            //            .insert(Transform::from_xyz(0., 0., 1.));
        }

        ew_atualiza_slot.send(e_atualiza_slot);
        ew_atualiza_jogador.send(e_atualiza_jogador {
            tipo: TipoAtualizacao::sobe_level,
            valor: 1,
        });
    }
}

#[derive(Event)]
struct e_resetar_jogo;

//um evento para zerar o level, colocar o jogador no lugar inicial e começar um jogo novo

#[derive(Event)]
struct e_monta_jogo;

fn montar_jogo(
    mut commands: Commands,
    mut res_deck: ResMut<config>,
    mut asset_server: Res<AssetServer>,
    mut q_camera: Query<(&mut Transform, &Camera2d), (Without<Carta>, With<Camera2d>)>,
) {
    res_deck.jogador.level = 0;
    res_deck.jogador.ataque = 1;
    res_deck.jogador.defesa = 1;
    res_deck.jogador.vida_atual = 10;
    res_deck.jogador.vida_inicial = 10;
    res_deck.jogador.ouro = 0;
    res_deck.jogador.xp = 0;
    res_deck.jogador.posicao = 1;
    res_deck.deck = Deck::default();
    res_deck.deck.init_de_json();
    res_deck.deck.monta_deck();
    let deck_img = asset_server.load("deck.png");
    let slot_img = asset_server.load("cemiterio.png");

    let mut deck = res_deck.deck.clone();

    for (mut transform, _) in q_camera.iter_mut() {
        transform.translation.y = 0.;
    }

    commands.spawn((
        //        PickableBundle::default(),
        deck.clone(),
        // On::<Pointer<Down>>::run(spawna_carta),
        SpriteBundle {
            texture: deck_img.clone(),
            transform: Transform::from_xyz(-450., 0., 0.).with_scale(Vec3::splat(1.0)),

            ..Default::default()
        },
    ));
    let mut slot = Slot::default();
    for ii in 0..3 {
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

                tipo: TipoCarta::Vazio,
                valor: Some(0),
            });
            slot.set_level(ii);
            slot.posicao = i - 1; //esta começando do 1
                                  //tween que faz as cartas irem da poscao do deck ate a posicao do slot que ela ficara
                                  //no tabuleiro

            commands.spawn((
                //       PickableBundle::default(),
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
                        (190. * i as f32) - 300.,
                        (255. * ii as f32) + 70.,
                        0.,
                    )
                    .with_scale(Vec3::splat(1.0)),

                    ..Default::default()
                },
            ));
        }
    }

    deck.level = 2;

    commands.spawn((
        //PickableBundle::default(),
        Slot::default(),
        SpriteBundle {
            //  sprite: Sprite::new(Vec2::new(100.0, 100.0)),
            texture: slot_img.clone(),
            transform: Transform::from_xyz(80., -195., 0.).with_scale(Vec3::splat(1.0)),

            ..Default::default()
        },
    ));

    //    ew_spawna_carta.send(e_spawnar_carta);
    let tween_heroi_deck_para_slot = Tween::new(
        EaseFunction::QuadraticInOut,
        Duration::from_millis(600),
        TransformPositionLens {
            start: Vec3::new(-200., 0., 0.),
            end: Vec3::new(80., -195., 9.),
        },
    );
    let carta_img: Handle<Image> = asset_server.load("carta-heroi.png");
    let carta_id = commands
        .spawn((
            Animator::new(tween_heroi_deck_para_slot),
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
            Ancora { x: 80., y: -195. },
            PickableBundle::default(),
            On::<Pointer<DragStart>>::target_insert(Dragging),
            On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                transform.translation.x += drag.delta.x;
                transform.translation.y -= drag.delta.y;
            }),
            On::<Pointer<DragEnd>>::target_insert(ProcessaFimDragging),
            SpriteBundle {
                texture: carta_img.clone(),
                transform: Transform::from_xyz(80., -195., 9.).with_scale(Vec3::splat(1.0)),
                ..Default::default()
            },
        ))
        .id();

    //spawnar os labels da carta
    let texto_heroi_vida = commands
        .spawn((
            UiCartaJogador {
                tipo: TipoUiCartaJogador::Vida,
            },
            LabelJogador,
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 38.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(-48., 81., 1.),
                ..Default::default()
            },
        ))
        .id();

    let texto_heroi_ataque = commands
        .spawn((
            UiCartaJogador {
                tipo: TipoUiCartaJogador::Ataque,
            },
            LabelJogador,
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 38.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(-48., 8., 1.),
                ..Default::default()
            },
        ))
        .id();

    let texto_heroi_defesa = commands
        .spawn((
            UiCartaJogador {
                tipo: TipoUiCartaJogador::Defesa,
            },
            LabelJogador,
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 38.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(-48., 44., 1.),
                ..Default::default()
            },
        ))
        .id();

    let texto_heroi_level = commands
        .spawn((
            UiCartaJogador {
                tipo: TipoUiCartaJogador::Level,
            },
            LabelJogador,
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 38.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(-48., -88., 1.),
                ..Default::default()
            },
        ))
        .id();
    let texto_heroi_ouro = commands
        .spawn((
            UiCartaJogador {
                tipo: TipoUiCartaJogador::Ouro,
            },
            LabelJogador,
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "0".to_string(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 38.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(40., -88., 1.),
                ..Default::default()
            },
        ))
        .id();

    commands
        .entity(carta_id)
        .push_children(&[texto_heroi_level]);
    commands.entity(carta_id).push_children(&[texto_heroi_ouro]);
    commands
        .entity(carta_id)
        .push_children(&[texto_heroi_ataque]);
    commands.entity(carta_id).push_children(&[texto_heroi_vida]);
    commands
        .entity(carta_id)
        .push_children(&[texto_heroi_defesa]);

    res_deck.deck = deck.clone();
    //    ew_atualiza_slot.send(e_atualiza_slot);
    //  info!("Jogo montado");
}

#[derive(Debug, Component, Clone)]
struct UiCartaJogador {
    tipo: TipoUiCartaJogador,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TipoUiCartaJogador {
    Vida,
    Defesa,
    Ataque,
    Level,
    Ouro,
    Xp,
}

fn resetar_jogo(
    mut commands: Commands,
    mut q_slots: Query<(Entity, &Slot)>,
    mut q_cartas: Query<(Entity, &Carta)>,
    mut q_texto_jogador: Query<(Entity, &Text), (Without<Status>, Without<UIEfeitosInventario>)>,
    mut q_deck: Query<(Entity, &Deck)>,
    mut config: ResMut<config>,
    mut q_camera: Query<(&mut OrthographicProjection), (Without<Carta>, With<Camera2d>)>,
    //  mut ew_monta_jogo: EventWriter<e_monta_jogo>,
    //    mut world: World,
) {
    config.deck = Deck::default();
    for mut op in q_camera.iter_mut() {
        op.scale = 1.2;
    }
    for (entity, slot) in q_slots.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for (entity, carta) in q_cartas.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for (entity, deck) in q_deck.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for (entidade, _) in q_texto_jogador.iter_mut() {
        //texto.sections[0].value = "".to_string();
        commands.entity(entidade).despawn_recursive();
    }

    // ew_monta_jogo.send(e_monta_jogo);
}

#[derive(Debug, Component, Clone)]
struct UIEfeitosInventario;

fn setup(mut commands: Commands, mut ew_resetar_jogo: EventWriter<e_resetar_jogo>) {
    let mut camera = commands.spawn(Camera2dBundle::default()).id();

    let status = commands
        .spawn((
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Status :".to_string(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 28.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(-540., 281., 11.),
                ..Default::default()
            },
            Status,
            //    Transform::from_xyz(300., 0., 1.),
        ))
        .id();
    // status.insert(Transform::from_xyz(300., 0., 1.));

    let efeitos_inventario = commands
        .spawn((
            Text2dBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Inventario/Efeitos".to_string(),
                        style: TextStyle {
                            //              font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 28.0,
                            color: Color::WHITE,
                            ..Default::default()
                        },
                    }],
                    ..Default::default()
                },
                transform: Transform::from_xyz(-540., 111., 11.),
                ..Default::default()
            },
            UIEfeitosInventario,
        ))
        .id();

    //efeitos_inventario.insert(Transform::from_xyz(300., 500., 1.));
    commands.entity(camera).push_children(&[status]);
    commands.entity(camera).push_children(&[efeitos_inventario]);
    ew_resetar_jogo.send(e_resetar_jogo);
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
                    transform: Transform::from_xyz(-100., 0., 1.).with_scale(Vec3::splat(1.0)),
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
