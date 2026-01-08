use bevy::prelude::*; // Bevy
use bevy::app::AppExit; // Used to close the app.
use bevy::window::PrimaryWindow; // Used to change the size of the screen.
use rand::Rng; // Used to generate food spawn position.


// Margin of the grid from the edge of the screen.
const MARGIN : i32 = 16;
// The number of cells in the grid. Can be changed freely.
const NUM_CELLS : i32 = 20;
// Size of the screen basically.
const WORLD_SIZE : i32 = 700;
// Size of the grid cells is calculated dynamically using the number of cells.
const CELL_SIZE : f32 = (WORLD_SIZE as f32 - MARGIN as f32 * 2.0) / NUM_CELLS as f32;
// How many seconds between the snake moving.
const MOVE_PERIOD : f32 = 0.15;
// Width of the outlines on the grids.
const OUTLINE_WIDTH : f32 = 1.0;
// Starting position of the snake.
const SNAKE_START_POS : GridPosition = GridPosition{x : NUM_CELLS / 2, y : NUM_CELLS / 2};
// Colors!!
const WHITE : Color = Color::srgb(1.0, 1.0, 1.0);
const BLACK : Color = Color::srgb(0.0, 0.0, 0.0);
const GREEN : Color = Color::srgb(0.25, 0.75, 0.25);
const RED : Color = Color::srgb(0.75, 0.25, 0.25);



// An enum that represents the possible directions.
#[derive(Copy, Clone, Eq, PartialEq)] // Allows us to use equality operators.
enum Direction { None, Up, Down, Left, Right }
impl Direction {
	// The values for each of the enumerations. These can be of mixed types by the way!!
	fn delta(self) -> (i32, i32) {
		match self {
			Direction::None => (0, 0),
			Direction::Up => (0, 1),
			Direction::Down => (0, -1),
			Direction::Left => (-1, 0),
			Direction::Right => (1, 0),
		}
	}
	
	// Determines if this direction is opposite of the other direction.
	// Used to determine if a player's movement input should be blocked or not.
	// Eg. If you're going up and press down, you shouldn't be able to go straight down.
	fn is_opposite(self, other : Direction) -> bool {
		matches!(
			(self, other),
			(Direction::Up, Direction::Down) | 
			(Direction::Down, Direction::Up) |
			(Direction::Left, Direction::Right) | 
			(Direction::Right, Direction::Left)
		)
	}
}



// The head of the snake. Used to sense collisions.
#[derive(Component)]
struct SnakeHead;

// A segment of a snake that isn't the head.
#[derive(Component)]
struct SnakeSegment;

// A position on the main grid, instead of using pixel locations.
#[derive(Component, Copy, Clone, Eq, PartialEq, Debug)]
struct GridPosition {
	x : i32,
	y : i32,
}

// The food object.
#[derive(Component)]
struct Food;

// The information about the snake, such as it's direction, references to all of its
// segments, and the number of segments that need to be added.
// Works similarly to a global/static variable, stores a single copy of the data.
#[derive(Resource)]
struct SnakeState {
	// The direction the snake is currently facing.
	dir : Direction,
	// The direction the snake should face on the next movement tick.
	next_dir : Direction,
	// References to the segments.
	segments : Vec<Entity>,
	// How many segments need to be added on the next tick.
	grow : u32,
}



// The method that is called when the program executes.
fn main() {
	/*
	So, what are systems? They are a special feature of Bevy that helps us implement
	the ECS structure. Instead of having normal functions, which are sent data when
	they're called somewhere else, systems are called only when scheduled, and are
	provided data from the ECS that matches their parameters.
	
	Eg. In the collision systems, they generally use the snake state, the head, and 
	then provide a query to fetch the segments of the snake. 
	
	This allows us to do the following:
	1) We use the position of the head compared to some other entity to see if it's
	   collided with something.
	2) If it has collided, we use the head and segment values in order to despawn all of
	   their entities.
	3) We update the properties of the snake state to be reset after the snake dies!
	*/
    App::new()
    	// Default plugins provide us base rendering, physics, etc.
        .add_plugins(DefaultPlugins)
        // Adding the SnakeState to the project so it can be accessed.
        // Kinda works like a global/static variable in a way.
        .insert_resource(SnakeState {
            dir : Direction::None,
            next_dir : Direction::None,
            segments : Vec::new(),
            grow : 0,
        })
        // Add the fixed timer that will be used when rendering objects and handle physics.
        .insert_resource(Time::<Fixed>::from_seconds(MOVE_PERIOD as f64))
        // Startup systems to initialize the game and spawn starting objects.
        .add_systems(Startup, (setup_camera_sys, 
        					   setup_screen_sys, 
        					   (spawn_grid_sys, (spawn_snake_sys, spawn_food_sys)).chain()))
        // Each frame we need to align objects to the grid and get the user's input.
        .add_systems(Update, (align_grid_to_world_sys, get_input_sys))
        // Allows us to close the game with the esc key.
        .add_systems(Update, exit_sys)
        // Everything else that should be updated when the timer loops.
        .add_systems(FixedUpdate, (move_snake_sys, 
        						   grow_snake_sys,
        						   wall_collision_sys, 
        						   food_collision_sys,
        						   snake_collision_sys))
        .run();
}



// Spawns the camera. Not much else to say lol.
fn setup_camera_sys(mut commands : Commands) {
	commands.spawn(Camera2d);
}



// Create the window and set its dimensions.
fn setup_screen_sys(mut windows : Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = windows.single_mut().unwrap();
    let size = WORLD_SIZE as f32;
    window.resolution.set(size, size);
}


// Basically takes all objects that have grid positions and moves them to the grid. 
// This system is only called when an entities grid position changes.
fn align_grid_to_world_sys(mut query : Query<(&GridPosition, &mut Transform), Changed<GridPosition>>) {
    for (grid_pos, mut transform) in query.iter_mut() {
        let x = -WORLD_SIZE as f32 / 2.0 + MARGIN as f32 + (grid_pos.x as f32 + 0.5) * CELL_SIZE;
        let y = -WORLD_SIZE as f32 / 2.0 + MARGIN as f32 + (grid_pos.y as f32 + 0.5) * CELL_SIZE;
        transform.translation = Vec3::new(x, y, transform.translation.z);
    }
}



// Creates the grid cells basically.
fn spawn_grid_sys(mut commands : Commands) {
	for row in 0..NUM_CELLS {
		for column in 0..NUM_CELLS {
			spawn_square_sys(&mut commands, row, column);
		}
	}
}



// Spawns a single cell at a provided grid position.
fn spawn_square_sys(
	commands : &mut Commands, 
	row : i32, 
	column : i32
) {
	commands.spawn((
		GridPosition { x : column, y : row },
		Transform::default(),
		Visibility::default(),
	))
	// Drawing the cell.
	.with_children(|parent| {
		// Black Outline
		parent.spawn((
			Sprite {
				color : BLACK,
				custom_size : Some(Vec2::splat(CELL_SIZE)),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 0.0),
		));
		// White Fill
		parent.spawn((
			Sprite {
				color : WHITE,
				custom_size : Some(Vec2::splat(CELL_SIZE - OUTLINE_WIDTH * 2.0)),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 1.0),
		));
	});
}



// Spawns the snake into the game at the starting position.
fn spawn_snake_sys(mut commands : Commands) {
	let _head = commands.spawn((
		SnakeHead, 
		SNAKE_START_POS,
		Transform::default(),
		Visibility::default(),
	))
	// Drawing the snake.
	.with_children(|parent| {
		// Black Outline
		parent.spawn((
			Sprite {
				color : BLACK,
				custom_size : Some(Vec2::splat(CELL_SIZE)),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 0.0),
		));
		// Green Fill
		parent.spawn((
			Sprite {
				color : GREEN,
				custom_size : Some(Vec2::splat(CELL_SIZE - OUTLINE_WIDTH * 4.0)),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 2.0),
		));
	});
}



// Spawns the food at a random position.
fn spawn_food_sys(mut commands : Commands) {
	let _food = commands.spawn((
		Food, 
		get_random_pos(),
		Transform::default(),
		Visibility::default(),
	))
	// Drawing the food.
	.with_children(|parent| {
		// Black Outline
		parent.spawn((
			Sprite {
				color : BLACK,
				custom_size : Some(Vec2::splat(CELL_SIZE)),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 0.0),
		));
		// Green Fill
		parent.spawn((
			Sprite {
				color : RED,
				custom_size : Some(Vec2::splat(CELL_SIZE - 12.0)),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 1.0),
		));
	});
}



// Gets a random cell position based on the number of cells in the grid.
fn get_random_pos() -> GridPosition {
	let col = rand::thread_rng().gen_range(0..NUM_CELLS);
	let row = rand::thread_rng().gen_range(0..NUM_CELLS);
    return GridPosition{ x : col, y : row };
}



// Takes input from the user! Does not include the esc key to exit.
fn get_input_sys(keyboard_input : Res<ButtonInput<KeyCode>>, mut snake : ResMut<SnakeState>) {
	if keyboard_input.pressed(KeyCode::ArrowUp) {
        if !snake.dir.is_opposite(Direction::Up) && snake.dir == snake.next_dir {
        	snake.next_dir = Direction::Up;
        }
    }
    else if keyboard_input.pressed(KeyCode::ArrowDown) {
    	if !snake.dir.is_opposite(Direction::Down) && snake.dir == snake.next_dir {
        	snake.next_dir = Direction::Down;
        }
    }
    else if keyboard_input.pressed(KeyCode::ArrowLeft) {
    	if !snake.dir.is_opposite(Direction::Left) && snake.dir == snake.next_dir {
        	snake.next_dir = Direction::Left;
        }
    }
    else if keyboard_input.pressed(KeyCode::ArrowRight) {
    	if !snake.dir.is_opposite(Direction::Right) && snake.dir == snake.next_dir {
        	snake.next_dir = Direction::Right;
        }
    }
}



// Moves the snake by first moving the head, and then, moving every segment after to the
// previous position of the segment in front of it.
fn move_snake_sys(
    mut snake : ResMut<SnakeState>,
    mut head_query : Query<&mut GridPosition, (With<SnakeHead>, Without<SnakeSegment>)>,
    mut seg_query : Query<&mut GridPosition, With<SnakeSegment>>,
) {
    // Move head
    let mut head_pos = head_query.single_mut().unwrap();
    snake.dir = snake.next_dir;
    let (dx, dy) = snake.dir.delta();
    let old_head_pos = *head_pos;
    head_pos.x += dx;
    head_pos.y += dy;

    // Move each segment to the previous position
    let mut prev_pos = old_head_pos;
    for &seg_entity in snake.segments.iter() {
        if let Ok(mut seg_pos) = seg_query.get_mut(seg_entity) {
            let current_pos = *seg_pos;
            *seg_pos = prev_pos;
            prev_pos = current_pos;
        }
    }
}



// Checks if the snake needs a new segment. If it does, we need to determine the position 
// and then spawn the new segment. Finally, decrement the grow property by one.
fn grow_snake_sys(
    mut commands : Commands,
    mut snake : ResMut<SnakeState>,
    seg_query : Query<&GridPosition, With<SnakeSegment>>,
    mut head_query : Query<&GridPosition, (With<SnakeHead>, Without<SnakeSegment>)>
) {
    if snake.grow == 0 {
        return;
    }

    // Determine spawn position by either the last segment of the snake, or the head if
    // there are no additional segments.
    let spawn_pos = if let Some(&tail_entity) = snake.segments.last() {
        *seg_query.get(tail_entity).unwrap()
    } else {
    	let head_pos = head_query.single_mut().unwrap();
        GridPosition {
            x : head_pos.x,
            y : head_pos.y,
        }
    };
    

    // Spawn new segment.
    let new_segment = commands
        .spawn((
            SnakeSegment,
            spawn_pos,
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|parent| {
            // Outline
            parent.spawn((
                Sprite {
                    color: BLACK,
                    custom_size: Some(Vec2::splat(CELL_SIZE)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
            ));
            // Fill
            parent.spawn((
                Sprite {
                    color: GREEN,
                    custom_size: Some(Vec2::splat(CELL_SIZE - OUTLINE_WIDTH * 4.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 1.0),
            ));
        })
        .id();

    // Add the new segment to the reference list, and decrement the grow counter.
    snake.segments.push(new_segment);
    snake.grow -= 1;
}



// Checks if the snake has "collided" with the wall (going out of bounds). If it does,
// resets the game by despawning the entities and resetting the properties.
fn wall_collision_sys(
	mut commands : Commands,
	mut snake : ResMut<SnakeState>,
	mut head : Query<(Entity, &mut GridPosition), (With<SnakeHead>, Changed<GridPosition>)>,
	segments : Query<Entity, With<SnakeSegment>>
) {
	let (head_entity, head_pos) = head.single_mut().unwrap();

	if head_pos.x < 0 || 
	   head_pos.x >= NUM_CELLS || 
	   head_pos.y < 0 || 
	   head_pos.y >= NUM_CELLS {
		snake.dir = Direction::None;
		snake.next_dir = Direction::None;
		snake.segments.clear();
		snake.grow = 0;
		
		commands.entity(head_entity).despawn();
		for seg_entity in segments.iter() {
    		commands.entity(seg_entity).despawn();
		}
		
		spawn_snake_sys(commands);
	}
}



// Checks if the snake has collided with a food, and tells the snake to grow if it has!
fn food_collision_sys(
	mut commands : Commands,
	mut snake : ResMut<SnakeState>,
	mut head : Query<(Entity, &GridPosition), With<SnakeHead>>,
	mut food : Query<(Entity, &GridPosition), With<Food>>
) {
	let (_head_entity, head_position) = head.single_mut().unwrap();
	let (food_entity, food_position) = food.single_mut().unwrap();
	
	if food_position == head_position {
		commands.entity(food_entity).despawn();
		spawn_food_sys(commands);
		snake.grow += 1;
	}
}



// Checks if the snake has "collided" with itself. If it has, resets the game by 
// despawning the entities and resetting the properties.
fn snake_collision_sys(
    mut commands : Commands,
    mut snake : ResMut<SnakeState>,
    mut head_query : Query<(Entity, &GridPosition), With<SnakeHead>>,
    seg_query : Query<(Entity, &GridPosition), With<SnakeSegment>>,
) {
    let (head_entity, head_pos) = head_query.single_mut().unwrap();

    // Check if the head collides with any segment.
    if seg_query.iter().any(|(_, seg_pos)| seg_pos == head_pos) {
        // Collect all segment entities.
        let seg_entities: Vec<Entity> = seg_query.iter().map(|(e, _)| e).collect();

        // Despawn everything at once.
        for e in seg_entities {
            commands.entity(e).despawn();
        }
        commands.entity(head_entity).despawn();

        // Reset snake state.
        snake.segments.clear();
        snake.dir = Direction::None;
        snake.next_dir = Direction::None;
        snake.grow = 0;

        // Spawn the new snake!
        spawn_snake_sys(commands);
    }
}



// Exits the game if the user presses the esc key!
fn exit_sys(
	keys : Res<ButtonInput<KeyCode>>, 
	mut exit : MessageWriter<AppExit>
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}
