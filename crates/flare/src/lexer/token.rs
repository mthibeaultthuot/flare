use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum TokenKind {
    #[token("kernel")]
    Kernel,
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("var")]
    Var,
    #[token("const")]
    Const,
    #[token("return")]
    Return,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("in")]
    In,
    #[token("where")]
    Where,
    #[token("type")]
    Type,
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,

    #[token("grid")]
    Grid,
    #[token("block")]
    Block,
    #[token("shared_memory")]
    SharedMemory,
    #[token("compute")]
    Compute,
    #[token("thread_idx")]
    ThreadIdx,
    #[token("block_idx")]
    BlockIdx,
    #[token("block_dim")]
    BlockDim,
    #[token("sync_threads")]
    SyncThreads,
    #[token("load_shared")]
    LoadShared,

    #[token("schedule")]
    Schedule,
    #[token("stream")]
    Stream,
    #[token("pipeline")]
    Pipeline,
    #[token("parallel")]
    Parallel,
    #[token("sync")]
    Sync,
    #[token("stage")]
    Stage,
    #[token("auto")]
    Auto,
    #[token("manual")]
    Manual,
    #[token("hints")]
    Hints,
    #[token("dynamic")]
    Dynamic,

    #[token("memory")]
    Memory,
    #[token("persistent")]
    Persistent,
    #[token("temporary")]
    Temporary,
    #[token("streaming")]
    Streaming,
    #[token("checkpoint")]
    Checkpoint,
    #[token("recompute")]
    Recompute,

    #[token("device")]
    Device,
    #[token("replicate")]
    Replicate,
    #[token("p2p_transfer")]
    P2PTransfer,
    #[token("all_reduce")]
    AllReduce,

    #[token("backend")]
    Backend,
    #[token("cuda")]
    Cuda,
    #[token("metal")]
    Metal,
    #[token("rocm")]
    Rocm,

    #[token("fuse")]
    Fuse,
    #[token("inline")]
    Inline,
    #[token("into")]
    Into,

    #[token("strategy")]
    Strategy,
    #[token("profile")]
    Profile,
    #[token("streams")]
    Streams,
    #[token("depth")]
    Depth,
    #[token("from")]
    From,
    #[token("to")]
    To,
    #[token("devices")]
    Devices,
    #[token("budget")]
    Budget,

    #[token("Tensor")]
    Tensor,
    #[token("Matrix")]
    Matrix,
    #[token("Vector")]
    Vector,
    #[token("i32")]
    I32,
    #[token("i64")]
    I64,
    #[token("u32")]
    U32,
    #[token("u64")]
    U64,
    #[token("f32")]
    F32,
    #[token("f64")]
    F64,
    #[token("bool")]
    Bool,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("max")]
    Max,
    #[token("min")]
    Min,
    #[token("product")]
    Product,

    #[token("@fusion_point")]
    FusionPoint,
    #[token("@fusable")]
    Fusable,
    #[token("@fusion_transform")]
    FusionTransform,
    #[token("@fused_kernel")]
    FusedKernel,
    #[token("@optimize")]
    Optimize,
    #[token("@auto_tune")]
    AutoTune,
    #[token("@schedule")]
    ScheduleAnnotation,
    #[token("@memory")]
    MemoryAnnotation,
    #[token("@depends_on")]
    DependsOn,
    #[token("@independent")]
    Independent,
    #[token("@prefer_parallel")]
    PreferParallel,
    #[token("@must_wait")]
    MustWait,
    #[token("@dynamic_dispatch")]
    DynamicDispatch,
    #[token("@pipeline_depth")]
    PipelineDepth,
    #[token("@p2p_transfer")]
    P2PTransferAnnotation,
    #[token("@all_reduce")]
    AllReduceAnnotation,

    #[token("@")]
    At,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,

    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,

    #[token("=")]
    Assign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,

    #[token("/=")]
    SlashAssign,

    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("|>")]
    Pipe,
    #[token(".")]
    Dot,
    #[token("..")]
    DotDot,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("?")]
    Question,
    
    
    #[token("...")]
    Ellipsis,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    IntLiteral(i64),
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),
    #[regex(r#""([^"\\]|\\["\\bnfrt]|u[a-fA-F0-9]{4})*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    StringLiteral(String),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[token("\n")]
    Newline,
    
    
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'src> {
    pub kind: TokenKind,
    pub idx: usize,
    pub text: &'src str,
    pub span: std::ops::Range<usize>,
}

impl<'src> Token<'src> {
    pub fn new(kind: TokenKind, idx: usize, text: &'src str, span: std::ops::Range<usize>) -> Self {
        Self {
            kind,
            idx,
            text,
            span,
        }
    }
}
