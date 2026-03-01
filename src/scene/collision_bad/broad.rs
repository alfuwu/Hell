use crate::scene::collision_bad::shapes::AABB;

#[derive(Clone)]
enum BvhNode {
    Leaf {
        object_idx: usize,
        aabb: AABB
    },
    Internal {
        aabb: AABB,
        left: Box<BvhNode>,
        right: Box<BvhNode>
    }
}
impl BvhNode {
    fn aabb(&self) -> &AABB {
        match self {
            BvhNode::Leaf { aabb, .. } => aabb,
            BvhNode::Internal { aabb, .. } => aabb
        }
    }

    fn query_pairs(&self, other_aabb: &AABB, other_idx: usize, out: &mut Vec<(usize, usize)>) {
        if !self.aabb().intersects(other_aabb) { return; }
        match self {
            BvhNode::Leaf { object_idx, .. } => {
                if *object_idx < other_idx {
                    out.push((*object_idx, other_idx));
                } else if *object_idx > other_idx {
                    out.push((other_idx, *object_idx));
                }
            }
            BvhNode::Internal { left, right, .. } => {
                left.query_pairs(other_aabb, other_idx, out);
                right.query_pairs(other_aabb, other_idx, out);
            }
        }
    }

    fn self_query(&self, out: &mut Vec<(usize, usize)>) {
        if let BvhNode::Internal { left, right, .. } = self {
            cross_query(left, right, out);
            left.self_query(out);
            right.self_query(out);
        }
    }
}

fn cross_query(a: &BvhNode, b: &BvhNode, out: &mut Vec<(usize, usize)>) {
    if !a.aabb().intersects(b.aabb()) { return; }
    match (a, b) {
        (BvhNode::Leaf { object_idx: ia, .. }, BvhNode::Leaf { object_idx: ib, .. }) => {
            let (lo, hi) = if ia < ib { (*ia, *ib) } else { (*ib, *ia) };
            out.push((lo, hi));
        },
        (BvhNode::Leaf { .. }, BvhNode::Internal { left, right, .. }) => {
            cross_query(a, left, out);
            cross_query(a, right, out);
        },
        (BvhNode::Internal { left, right, .. }, BvhNode::Leaf { .. }) => {
            cross_query(left, b, out);
            cross_query(right, b, out);
        },
        (BvhNode::Internal { left: la, right: ra, .. }, BvhNode::Internal { left: lb, right: rb, .. }) => {
            cross_query(la, lb, out);
            cross_query(la, rb, out);
            cross_query(ra, lb, out);
            cross_query(ra, rb, out);
        }
    }
}

pub struct Bvh {
    root: Option<BvhNode>
}
impl Bvh {
    pub fn build(items: &[(usize, AABB)]) -> Self {
        if items.is_empty() { return Self { root: None }; }
        let root = build_node(items);
        Self { root: Some(root) }
    }

    pub fn candidate_pairs(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.self_query(&mut out);
        }
        out.sort_unstable();
        out.dedup();
        out
    }
}

fn build_node(items: &[(usize, AABB)]) -> BvhNode {
    if items.len() == 1 {
        return BvhNode::Leaf { object_idx: items[0].0, aabb: items[0].1.clone() };
    }

    let total = items.iter().fold(items[0].1.clone(), |acc, (_, a)| acc.union(a));

    if items.len() == 2 {
        let left  = Box::new(BvhNode::Leaf { object_idx: items[0].0, aabb: items[0].1.clone() });
        let right = Box::new(BvhNode::Leaf { object_idx: items[1].0, aabb: items[1].1.clone() });
        return BvhNode::Internal { aabb: total, left, right };
    }

    let extent = total.max - total.min;
    let axis = if extent.x >= extent.y && extent.x >= extent.z { 0 }
    else if extent.y >= extent.z { 1 }
    else { 2 };

    let mut sorted: Vec<(usize, AABB)> = items.to_vec();
    sorted.sort_unstable_by(|(_, a), (_, b)| {
        let ca = a.center()[axis];
        let cb = b.center()[axis];
        ca.partial_cmp(&cb).unwrap()
    });

    let mid = sorted.len() / 2;
    let left  = Box::new(build_node(&sorted[..mid]));
    let right = Box::new(build_node(&sorted[mid..]));
    BvhNode::Internal { aabb: total, left, right }
}