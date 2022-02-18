use puker::{
    player::*,
    consts::*,
    utils::*,
    items::*,
};

#[test]
fn test_player_pick_up_collectable() {
    let mut player = Player::default();
    player.health = 2.5;

    let mut collectable = Collectable {
        props: ActorProps::default(),
        tag: CollectableTag::RedHeart(0.5),
        state: CollectableState::Base,
    };

    collectable.affect_player(&mut player);
    assert_eq!(player.health, 3.);
    assert_eq!(collectable.state, CollectableState::Consumed);

    collectable.state = CollectableState::Base;
    collectable.affect_player(&mut player);
    assert_eq!(player.health, 3.);
    assert_eq!(collectable.state, CollectableState::Base);

    collectable.tag = CollectableTag::SpeedBoost(1.1);
    collectable.affect_player(&mut player);
    assert_eq!(player.speed, PLAYER_SPEED * 1.1);
    assert_eq!(collectable.state, CollectableState::Consumed);

    collectable.state = CollectableState::Base;
    collectable.tag = CollectableTag::ShootRateBoost(1.1);
    collectable.affect_player(&mut player);
    assert_eq!(player.shoot_rate, PLAYER_SHOOT_RATE * 1.1);
    assert_eq!(collectable.state, CollectableState::Consumed);

    collectable.state = CollectableState::Base;
    collectable.tag = CollectableTag::DamageBoost(1.1);
    collectable.affect_player(&mut player);
    assert_eq!(player.damage, PLAYER_DAMAGE * 1.1);
    assert_eq!(collectable.state, CollectableState::Consumed);
}

#[test]
fn test_player_pick_up_passive() {
    let mut player = Player::default();
    let mut passive = Item {
        tag: ItemTag::Passive(ItemPassive::IncreaseMaxHealth(1.)),
        cooldown: 0.,
    };
    
    passive.affect_player(&mut player);
    assert_eq!(player.max_health, 4.);
}

#[test]
fn test_player_use_item() {
    let mut player = Player::default();

    assert!(!player.use_item());

    player.item = Some(Item {
        tag: ItemTag::Active(ItemActive::Heal(1.)),
        cooldown: 0.,
    });
    player.health = 2.;

    assert!(player.use_item());
    assert!(!player.use_item());
    assert_eq!(player.health, 3.);
    assert_eq!(player.item.unwrap().cooldown, ITEM_COOLDOWN);

    player.item.as_mut().unwrap().cooldown = 0.;
    assert!(player.use_item());
    assert_eq!(player.health, 3.);
    assert_eq!(player.item.unwrap().cooldown, ITEM_COOLDOWN);
}
