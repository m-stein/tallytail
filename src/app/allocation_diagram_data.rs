pub struct AllocationDiagramSegment {
    pub name: Option<String>,
    pub amount: f64,
}

pub struct AllocationDiagramBar {
    pub date: String,
    pub segments: Vec<AllocationDiagramSegment>,
}

pub struct AllocationDiagramData {
    pub title: String,
    pub bars: Vec<AllocationDiagramBar>,
}