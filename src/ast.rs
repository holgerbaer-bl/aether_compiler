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
    Eq(Box<Node>, Box<Node>),
    Lt(Box<Node>, Box<Node>),

    // Arrays & Strings
    ArrayLiteral(Vec<Node>),
    ArrayGet(String, Box<Node>),            // Variable, Index
    ArraySet(String, Box<Node>, Box<Node>), // Variable, Index, Value
    ArrayPush(String, Box<Node>),           // Variable, Value
    ArrayLen(String),                       // Variable
    Index(Box<Node>, Box<Node>),            // General index (Expression based)
    Concat(Box<Node>, Box<Node>),

    // Bitwise
    BitAnd(Box<Node>, Box<Node>),
    BitShiftLeft(Box<Node>, Box<Node>),
    BitShiftRight(Box<Node>, Box<Node>),

    // Functions
    FnDef(String, Vec<String>, Box<Node>),
    Call(String, Vec<Node>),

    // I/O
    FileRead(Box<Node>),
    FileWrite(Box<Node>, Box<Node>),
    Print(Box<Node>),

    // FFI / Reflection
    EvalJSONNative(Box<Node>),
    ToString(Box<Node>),
    NativeCall(String, Vec<Node>), // Function Name, Args

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
    UIWindow(Box<Node>, Box<Node>), // Title (String), Children (Block)
    UILabel(Box<Node>),             // Text (String)
    UIButton(Box<Node>),            // Text (String). Evaluates to Boolean (Clicked)
    UITextInput(Box<Node>),         // Variable Name to bind to (String)

    // Voxel Engine (Sprint 12 & 13)
    InitCamera(Box<Node>),    // FOV (Float). Activates 3D FPS camera
    DrawVoxelGrid(Box<Node>), // Array of Positions (XYZ layout)
    LoadTextureAtlas(Box<Node>, Box<Node>), // Path (String), TileSize (Float)
    LoadSample(Box<Node>, Box<Node>), // ID (Int), Path (String)
    PlaySample(Box<Node>, Box<Node>, Box<Node>), // ID (Int), Volume (Float), Pitch (Float)
    InitVoxelMap,             // Transfers Voxel control to a mutable HashMap
    SetVoxel(Box<Node>, Box<Node>, Box<Node>, Box<Node>), // X, Y, Z, ID
    EnableInteraction(Box<Node>), // Boolean (True): Activates Raycasting & Mouse Mapping

    // Control Flow
    If(Box<Node>, Box<Node>, Option<Box<Node>>),
    While(Box<Node>, Box<Node>),
    Block(Vec<Node>),
    Return(Box<Node>),
}
