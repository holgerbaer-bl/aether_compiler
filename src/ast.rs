use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    BoolLiteral(bool),
    StringLiteral(String),

    // Memory
    Identifier(String),
    Assign(String, Box<Node>),

    // Math & Logic
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Sin(Box<Node>),
    Cos(Box<Node>),
    Mat4Mul(Box<Node>, Box<Node>),
    Time,
    GlobalTime,
    Abs(Box<Node>),
    Eq(Box<Node>, Box<Node>),
    Lt(Box<Node>, Box<Node>),
    Gt(Box<Node>, Box<Node>),

    // Arrays, Strings, Objects & Maps
    ArrayCreate(Vec<Node>),
    ArrayGet(Box<Node>, Box<Node>),            // Target Array, Index
    ArraySet(Box<Node>, Box<Node>, Box<Node>), // Target Array, Index, Value
    ArrayPush(Box<Node>, Box<Node>),           // Target Array, Value
    ArrayLen(Box<Node>),                       // Target Array
    MapCreate,
    MapGet(Box<Node>, Box<Node>),            // Target Map, Key
    MapSet(Box<Node>, Box<Node>, Box<Node>), // Target Map, Key, Value
    MapHasKey(Box<Node>, Box<Node>),         // Target Map, Key
    Index(Box<Node>, Box<Node>),             // General index (Expression based)
    Concat(Box<Node>, Box<Node>),

    ObjectLiteral(std::collections::HashMap<String, Node>),
    PropertyGet(Box<Node>, String), // Target Object, Property Name
    PropertySet(Box<Node>, String, Box<Node>), // Target Object, Property Name, Value

    // Bitwise
    BitAnd(Box<Node>, Box<Node>),
    BitShiftLeft(Box<Node>, Box<Node>),
    BitShiftRight(Box<Node>, Box<Node>),

    // Functions
    FnDef(String, Vec<String>, Box<Node>),
    Call(String, Vec<Node>),

    // I/O & System Nodes (Sprint 59 extensions)
    FileRead(Box<Node>),
    FileWrite(Box<Node>, Box<Node>),
    Print(Box<Node>),
    FSRead(Box<Node>),             // Specialized Agent I/O
    FSWrite(Box<Node>, Box<Node>), // Specialized Agent I/O

    // AST Erweiterung für universelle Persistenz (Sprint 64)
    Store { key: String, value: Box<Node> },
    Load { key: String },

    // Sprint 67: Native 2D Drawing Primitives
    DrawRect {
        x: Box<Node>,
        y: Box<Node>,
        width: Box<Node>,
        height: Box<Node>,
        color: Box<Node>, // Array [R, G, B, A] as floats 0.0..1.0
    },
    UIFixed {
        width: Box<Node>,
        height: Box<Node>,
        body: Box<Node>,
    },
    UIFillParent,

    // Sprint 68: Native 3D/2D Render Scene Graph
    RenderCanvas { body: Box<Node> },
    Transform2D {
        x: Box<Node>,
        y: Box<Node>,
        rotation: Box<Node>,
        scale: Box<Node>,
        body: Box<Node>,
    },
    Sprite2D { texture_id: Box<Node>, transform: Box<Node> },
    Camera3D {
        pos_x: Box<Node>,
        pos_y: Box<Node>,
        pos_z: Box<Node>,
        target_x: Box<Node>,
        target_y: Box<Node>,
        target_z: Box<Node>,
        fov: Box<Node>,
    },
    Mesh3D { primitive: Box<Node>, material: Box<Node> }, // primitive: "cube"|"sphere"|"plane"
    PointLight3D {
        x: Box<Node>, y: Box<Node>, z: Box<Node>,
        r: Box<Node>, g: Box<Node>, b: Box<Node>,
        intensity: Box<Node>,
    },
    Material3D {
        r: Box<Node>, g: Box<Node>, b: Box<Node>, a: Box<Node>,
        metallic: Box<Node>, roughness: Box<Node>,
        texture_id: Option<Box<Node>>,
    },
    // Sprint 71: PS3-Era FPS Foundation
    MeshInstance3D {
        mesh_id: Box<Node>,
        transform: Box<Node>,
        color_offset: Box<Node>,
        pbr: Box<Node>,
    },
    FPSCamera {
        fov: Box<Node>,
    },
    MouseGrab {
        enabled: Box<Node>,
    },
    RaycastSimple,
    WeaponViewModel {
        mesh: Box<Node>,
        tex: Box<Node>,
    },

    // Sprint 60: Async Connectivity
    Fetch {
        method: String,
        url: String,
        callback: Box<Node>,
    },
    Extract {
        source: Box<Node>,
        path: Box<Node>,
    },

    // FFI / Reflection
    EvalJSONNative(Box<Node>),
    ToString(Box<Node>),
    NativeCall(String, Vec<Node>), // Function Name, Args
    ExternCall {
        module: String,
        function: String,
        args: Vec<Node>,
    }, // Foreign C/Rust Function

    // 3D Graphics (WGPU FFI)
    InitWindow(Box<Node>, Box<Node>, Box<Node>), // W, H, Title
    InitGraphics,                                // Bootstraps WGPU context
    LoadShader(Box<Node>),                       // WGSL string
    RenderMesh(Box<Node>, Box<Node>, Box<Node>), // Shader ID, Vertices, Uniform MVP Matrix
    PollEvents(Box<Node>),                       // Execution loop intercept

    // Audio Engine (CPAL FFI)
    InitAudio,
    PlayNote(Box<Node>, Box<Node>, Box<Node>), // Channel, Frequency, Waveform
    StopNote(Box<Node>),                       // Channel

    // Asset Pipeline (Sprint 7)
    LoadMesh(Box<Node>),                                     // Path String
    LoadTexture(Box<Node>),                                  // Path String
    PlayAudioFile(Box<Node>),                                // Path String
    RenderAsset(Box<Node>, Box<Node>, Box<Node>, Box<Node>), // Shader ID, Mesh ID, Texture ID, Uniform Matrix

    // UI & Text Engine (Sprint 8)
    LoadFont(Box<Node>), // Path String
    DrawText(Box<Node>, Box<Node>, Box<Node>, Box<Node>, Box<Node>), // Text String, X Float, Y Float, Size Float, Color Array[R,G,B,A]
    GetLastKeypress,                                                 // Returns String buffer

    // Egui UI
    UIWindow(String, Box<Node>, Box<Node>), // ID (String), Title (StringNode), Children (Block)
    UILabel(Box<Node>),                     // Text (String)
    UIButton(Box<Node>), // Text (String). Evaluates to Bool (true if clicked this frame)
    UITextInput(Box<Node>), // Variable Name to bind to (String)
    UISetStyle(
        Box<Node>,
        Box<Node>,
        Box<Node>,
        Box<Node>,
        Option<Box<Node>>,
        Option<Box<Node>>,
    ), // Rounding, Spacing, Accent RGBA, Fill RGBA, Button Idle RGBA (opt), Button Hover RGBA (opt)
    UIHorizontal(Box<Node>), // Render children side-by-side (horizontal layout)
    UIFullscreen(Box<Node>), // Render children in a full-canvas borderless panel
    UIGrid(i64, String, Box<Node>), // Columns, ID, Body
    UIScrollArea(String, Box<Node>), // ID, Body for native scrolling view

    // Voxel Engine (Sprint 12 & 13)
    InitCamera(Box<Node>),    // FOV (Float). Activates 3D FPS camera
    DrawVoxelGrid(Box<Node>), // Array of Positions (XYZ layout)
    LoadTextureAtlas(Box<Node>, Box<Node>), // Path (String), TileSize (Float)
    LoadSample(Box<Node>, Box<Node>), // ID (Int), Path (String)
    PlaySample(Box<Node>, Box<Node>, Box<Node>), // ID (Int), Volume (Float), Pitch (Float)
    InitVoxelMap,             // Transfers Voxel control to a mutable HashMap
    SetVoxel(Box<Node>, Box<Node>, Box<Node>, Box<Node>), // X, Y, Z, ID
    EnableInteraction(Box<Node>), // Boolean (True): Activates Raycasting & Mouse Mapping
    EnablePhysics(Box<Node>), // Boolean (True): Activates Gravity & Collision
    // Control Flow
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    Block(Vec<Node>),
    Return(Box<Node>),
    Import(String),
    AddWorldAABB { min: Box<Node>, max: Box<Node> },
    CheckCollision { a_min: Box<Node>, a_max: Box<Node>, b_min: Box<Node>, b_max: Box<Node> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    Array(Vec<Type>),
    Map(Box<Type>),
    Object,
    Handle,
    Any,
    Void,
}
