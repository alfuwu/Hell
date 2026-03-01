use crate::scene::collision_bad::collider::{WorldBox, WorldCapsule, WorldCollider, WorldConvexMesh, WorldSphere, WorldTriangle, WorldTriangleMesh};
use crate::util::vectors::Vector3f;

#[derive(Clone)]
pub struct Contact {
    pub normal: Vector3f,
    pub depth: f32,
    pub point: Vector3f
}

pub fn test(a: &WorldCollider, b: &WorldCollider) -> Option<Contact> {
    match (a, b) {
        (WorldCollider::Sphere(sa), WorldCollider::Sphere(sb))         => sphere_sphere(sa, sb),
        (WorldCollider::Sphere(s),  WorldCollider::Box(bx))            => sphere_box(s, bx),
        (WorldCollider::Box(bx),    WorldCollider::Sphere(s))          => sphere_box(s, bx).map(flip),
        (WorldCollider::Sphere(s),  WorldCollider::Capsule(c))         => sphere_capsule(s, c),
        (WorldCollider::Capsule(c), WorldCollider::Sphere(s))          => sphere_capsule(s, c).map(flip),
        (WorldCollider::Capsule(ca),WorldCollider::Capsule(cb))        => capsule_capsule(ca, cb),
        (WorldCollider::Box(ba),    WorldCollider::Box(bb))            => obb_obb(ba, bb),
        (WorldCollider::Box(bx),    WorldCollider::Capsule(c))         => box_capsule(bx, c),
        (WorldCollider::Capsule(c), WorldCollider::Box(bx))            => box_capsule(bx, c).map(flip),
        (WorldCollider::ConvexMesh(m), WorldCollider::Sphere(s))       => convex_sphere(m, s),
        (WorldCollider::Sphere(s),  WorldCollider::ConvexMesh(m))      => convex_sphere(m, s).map(flip),
        (WorldCollider::ConvexMesh(ma),WorldCollider::ConvexMesh(mb))  => convex_convex(ma, mb),
        (WorldCollider::ConvexMesh(m), WorldCollider::Box(bx))         => convex_obb(m, bx),
        (WorldCollider::Box(bx),    WorldCollider::ConvexMesh(m))      => convex_obb(m, bx).map(flip),
        (WorldCollider::ConvexMesh(m), WorldCollider::Capsule(c))      => convex_capsule(m, c),
        (WorldCollider::Capsule(c), WorldCollider::ConvexMesh(m))      => convex_capsule(m, c).map(flip),

        (WorldCollider::Box(bx),    WorldCollider::TriangleMesh(tm))        => triangle_mesh_obb(tm, bx),
        (WorldCollider::TriangleMesh(tm), WorldCollider::Box(bx))           => triangle_mesh_obb(tm, bx).map(flip),
        (WorldCollider::Sphere(s),  WorldCollider::TriangleMesh(tm))        => triangle_mesh_sphere(tm, s),
        (WorldCollider::TriangleMesh(tm), WorldCollider::Sphere(s))         => triangle_mesh_sphere(tm, s).map(flip),
        (WorldCollider::Capsule(c), WorldCollider::TriangleMesh(tm))        => triangle_mesh_capsule(tm, c),
        (WorldCollider::TriangleMesh(tm), WorldCollider::Capsule(c))        => triangle_mesh_capsule(tm, c).map(flip),
        // todo
        (WorldCollider::TriangleMesh(_), WorldCollider::TriangleMesh(_))    => None,
        (WorldCollider::ConvexMesh(_),   WorldCollider::TriangleMesh(_))    => None,
        (WorldCollider::TriangleMesh(_), WorldCollider::ConvexMesh(_))      => None,
    }
}

fn flip(mut c: Contact) -> Contact { c.normal = -c.normal; c }

fn closest_point_on_segment(p: Vector3f, q: &Vector3f, t: &Vector3f) -> (Vector3f, f32) {
    let pq = q - p;
    let len_sq = pq.length_squared();
    if len_sq < 1e-10 { return (p, 0.0); }
    let param = ((t - p).dot(&pq) / len_sq).clamp(0.0, 1.0);
    (p + pq * param, param)
}

fn closest_points_segment_segment(
    p1: &Vector3f, q1: &Vector3f,
    p2: &Vector3f, q2: &Vector3f,
) -> (Vector3f, Vector3f) {
    let d1 = q1 - p1;
    let d2 = q2 - p2;
    let r  = p1 - p2;
    let a  = d1.length_squared();
    let e  = d2.length_squared();
    let f  = d2.dot(&r);

    let (s, t) = if a < 1e-10 && e < 1e-10 {
        (0.0_f32, 0.0_f32)
    } else if a < 1e-10 {
        (0.0, (f / e).clamp(0.0, 1.0))
    } else {
        let c = d1.dot(&r);
        if e < 1e-10 {
            ((-c / a).clamp(0.0, 1.0), 0.0)
        } else {
            let b  = d1.dot(&d2);
            let denom = a * e - b * b;
            let s = if denom.abs() > 1e-10 {
                ((b * f - c * e) / denom).clamp(0.0, 1.0)
            } else { 0.0 };
            let t = (b * s + f) / e;
            if t < 0.0 {
                ((-c / a).clamp(0.0, 1.0), 0.0)
            } else if t > 1.0 {
                (((b - c) / a).clamp(0.0, 1.0), 1.0)
            } else {
                (s, t)
            }
        }
    };

    (p1 + d1 * s, p2 + d2 * t)
}

fn closest_point_on_obb(p: &Vector3f, obb: &WorldBox) -> Vector3f {
    let d = p - obb.center;
    let mut result = obb.center;
    for i in 0..3 {
        let dist = d.dot(&obb.axes[i]).clamp(
            -obb.half_extents[i],
            obb.half_extents[i],
        );
        result = result + obb.axes[i] * dist;
    }
    result
}

fn obb_project_half(obb: &WorldBox, axis: &Vector3f) -> f32 {
    obb.half_extents.x * obb.axes[0].dot(axis).abs()
        + obb.half_extents.y * obb.axes[1].dot(axis).abs()
        + obb.half_extents.z * obb.axes[2].dot(axis).abs()
}

fn sphere_sphere(a: &WorldSphere, b: &WorldSphere) -> Option<Contact> {
    let diff   = a.center - b.center;
    let dist_sq = diff.length_squared();
    let radii   = a.radius + b.radius;
    if dist_sq >= radii * radii { return None; }
    let dist = dist_sq.sqrt();
    let normal = if dist < 1e-6 { Vector3f::Y } else { diff / dist };
    Some(Contact {
        normal,
        depth: radii - dist,
        point: b.center + normal * b.radius,
    })
}

fn sphere_box(s: &WorldSphere, bx: &WorldBox) -> Option<Contact> {
    let closest = closest_point_on_obb(&s.center, bx);
    let diff    = s.center - closest;
    let dist_sq = diff.length_squared();
    if dist_sq >= s.radius * s.radius { return None; }
    let dist    = dist_sq.sqrt();
    let normal  = if dist < 1e-6 {
        let d = s.center - bx.center;
        let mut best_axis = bx.axes[0];
        let mut best_depth = f32::MAX;
        for i in 0..3 {
            let proj = d.dot(&bx.axes[i]);
            let depth = bx.half_extents[i] - proj.abs();
            if depth < best_depth {
                best_depth = depth;
                best_axis = if proj >= 0.0 { bx.axes[i] } else { -bx.axes[i] };
            }
        }
        best_axis
    } else {
        diff / dist
    };
    Some(Contact {
        normal,
        depth: s.radius - dist,
        point: closest,
    })
}

fn sphere_capsule(s: &WorldSphere, c: &WorldCapsule) -> Option<Contact> {
    let (closest, _) = closest_point_on_segment(c.tip_a, &c.tip_b, &s.center);
    let diff    = s.center - closest;
    let dist_sq = diff.length_squared();
    let radii   = s.radius + c.radius;
    if dist_sq >= radii * radii { return None; }
    let dist    = dist_sq.sqrt();
    let normal  = if dist < 1e-6 { Vector3f::Y } else { diff / dist };
    Some(Contact {
        normal,
        depth: radii - dist,
        point: closest + normal * c.radius,
    })
}

fn capsule_capsule(a: &WorldCapsule, b: &WorldCapsule) -> Option<Contact> {
    let (pa, pb) = closest_points_segment_segment(&a.tip_a, &a.tip_b, &b.tip_a, &b.tip_b);
    let diff     = pa - pb;
    let dist_sq  = diff.length_squared();
    let radii    = a.radius + b.radius;
    if dist_sq >= radii * radii { return None; }
    let dist     = dist_sq.sqrt();
    let normal   = if dist < 1e-6 { Vector3f::Y } else { diff / dist };
    Some(Contact {
        normal,
        depth: radii - dist,
        point: pb + normal * b.radius,
    })
}

fn obb_obb(a: &WorldBox, b: &WorldBox) -> Option<Contact> {
    let center_diff = b.center - a.center;
    let mut min_depth = f32::MAX;
    let mut best_axis = Vector3f::ZERO;

    macro_rules! test_axis {
        ($axis:expr) => {{
            let axis: Vector3f = $axis;
            let len_sq = axis.length_squared();
            if len_sq < 1e-10 { /* skip degenerate */ } else {
                let axis = axis / len_sq.sqrt();
                let ha = obb_project_half(a, &axis);
                let hb = obb_project_half(b, &axis);
                let dist = center_diff.dot(&axis).abs();
                let depth = ha + hb - dist;
                if depth < 0.0 { return None; }
                if depth < min_depth {
                    min_depth = depth;
                    best_axis = if center_diff.dot(&axis) >= 0.0 { -axis } else { axis };
                }
            }
        }};
    }

    for i in 0..3 { test_axis!(a.axes[i]); }
    for i in 0..3 { test_axis!(b.axes[i]); }
    for i in 0..3 {
        for j in 0..3 {
            test_axis!(a.axes[i].cross(&b.axes[j]));
        }
    }

    let point = (a.center + b.center) * 0.5;
    Some(Contact { normal: best_axis, depth: min_depth, point })
}

fn box_capsule(bx: &WorldBox, cap: &WorldCapsule) -> Option<Contact> {
    let ca = closest_point_on_obb(&cap.tip_a, bx);
    let cb = closest_point_on_obb(&cap.tip_b, bx);
    let (pa, _) = closest_point_on_segment(cap.tip_a, &cap.tip_b, &ca);
    let (pb, _) = closest_point_on_segment(cap.tip_a, &cap.tip_b, &cb);

    let (seg_pt, box_pt) = {
        let da = (pa - ca).length_squared();
        let db = (pb - cb).length_squared();
        if da <= db { (pa, ca) } else { (pb, cb) }
    };

    let diff    = seg_pt - box_pt;
    let dist_sq = diff.length_squared();
    if dist_sq >= cap.radius * cap.radius { return None; }

    let dist   = dist_sq.sqrt();
    let normal = if dist < 1e-6 { Vector3f::Y } else { diff / dist };
    Some(Contact {
        normal,
        depth: cap.radius - dist,
        point: box_pt,
    })
}

fn convex_sphere(mesh: &WorldConvexMesh, s: &WorldSphere) -> Option<Contact> {
    let mut min_depth = f32::MAX;
    let mut best_normal = Vector3f::ZERO;

    for &n in &mesh.face_normals {
        let support = mesh.support(-n);
        let dist    = (s.center - support).dot(&n) - s.radius;
        // dist < 0 means overlap on this axis
        if dist > 0.0 { return None; }
        let depth = -dist;
        if depth < min_depth {
            min_depth = depth;
            best_normal = n;
        }
    }

    let point = s.center - best_normal * s.radius;
    Some(Contact { normal: best_normal, depth: min_depth, point })
}

fn convex_obb(mesh: &WorldConvexMesh, bx: &WorldBox) -> Option<Contact> {
    let mut min_depth = f32::MAX;
    let mut best_normal = Vector3f::ZERO;

    macro_rules! test_axis {
        ($axis:expr) => {{
            let axis: Vector3f = $axis;
            let len_sq = axis.length_squared();
            if len_sq > 1e-10 {
                let axis = axis / len_sq.sqrt();
                let (mn_m, mx_m) = project_mesh(mesh, &axis);
                let c   = bx.center.dot(&axis);
                let h   = obb_project_half(bx, &axis);
                let (mn_b, mx_b) = (c - h, c + h);
                let overlap = mx_m.min(mx_b) - mn_m.max(mn_b);
                if overlap < 0.0 { return None; }
                if overlap < min_depth {
                    min_depth = overlap;
                    let mesh_c = mesh.center.dot(&axis);
                    best_normal = if mesh_c < c { axis } else { -axis };
                }
            }
        }};
    }

    for &n in &mesh.face_normals { test_axis!(n); }
    for i in 0..3 { test_axis!(bx.axes[i]); }
    for e in &mesh.edges {
        let edge_dir = (mesh.vertices[e[1]] - mesh.vertices[e[0]]).normalized();
        for i in 0..3 { test_axis!(edge_dir.cross(&bx.axes[i])); }
    }

    let point = (mesh.center + bx.center) * 0.5;
    Some(Contact { normal: best_normal, depth: min_depth, point })
}

fn convex_capsule(mesh: &WorldConvexMesh, cap: &WorldCapsule) -> Option<Contact> {
    let cap_dir = cap.axis();
    let mut min_depth = f32::MAX;
    let mut best_normal = Vector3f::ZERO;

    macro_rules! test_axis {
        ($axis:expr) => {{
            let axis: Vector3f = $axis;
            let len_sq = axis.length_squared();
            if len_sq > 1e-10 {
                let axis = axis / len_sq.sqrt();
                let (mn_m, mx_m) = project_mesh(mesh, &axis);
                let pa = cap.tip_a.dot(&axis);
                let pb = cap.tip_b.dot(&axis);
                let (mn_c, mx_c) = (pa.min(pb) - cap.radius, pa.max(pb) + cap.radius);
                let overlap = mx_m.min(mx_c) - mn_m.max(mn_c);
                if overlap < 0.0 { return None; }
                if overlap < min_depth {
                    min_depth = overlap;
                    let mesh_c = mesh.center.dot(&axis);
                    //let cap_c  = ((pa + pb) * 0.5).dot(&axis);
                    let cap_mid = ((cap.tip_a + cap.tip_b) * 0.5).dot(&axis);
                    best_normal = if mesh_c < cap_mid { axis } else { -axis };
                }
            }
        }};
    }

    for &n in &mesh.face_normals { test_axis!(n); }
    for e in &mesh.edges {
        let edge_dir = (mesh.vertices[e[1]] - mesh.vertices[e[0]]).normalized();
        test_axis!(edge_dir.cross(&cap_dir));
    }

    let point = (mesh.center + (cap.tip_a + cap.tip_b) * 0.5) * 0.5;
    Some(Contact { normal: best_normal, depth: min_depth, point })
}

fn convex_convex(a: &WorldConvexMesh, b: &WorldConvexMesh) -> Option<Contact> {
    let mut min_depth = f32::MAX;
    let mut best_normal = Vector3f::ZERO;

    macro_rules! test_axis {
        ($axis:expr) => {{
            let axis: Vector3f = $axis;
            let len_sq = axis.length_squared();
            if len_sq > 1e-10 {
                let axis = axis / len_sq.sqrt();
                let (mna, mxa) = project_mesh(a, &axis);
                let (mnb, mxb) = project_mesh(b, &axis);
                let overlap = mxa.min(mxb) - mna.max(mnb);
                if overlap < 0.0 { return None; }
                if overlap < min_depth {
                    min_depth = overlap;
                    let ca = a.center.dot(&axis);
                    let cb = b.center.dot(&axis);
                    best_normal = if ca < cb { axis } else { -axis };
                }
            }
        }};
    }

    for &n in &a.face_normals { test_axis!(n); }
    for &n in &b.face_normals { test_axis!(n); }
    for ea in &a.edges {
        for eb in &b.edges {
            let da = (a.vertices[ea[1]] - a.vertices[ea[0]]).normalized();
            let db = (b.vertices[eb[1]] - b.vertices[eb[0]]).normalized();
            test_axis!(da.cross(&db));
        }
    }

    let point = (a.center + b.center) * 0.5;
    Some(Contact { normal: best_normal, depth: min_depth, point })
}

pub fn triangle_vs_obb(tri: &WorldTriangle, obb: &WorldBox) -> Option<Contact> {
    let mut min_depth = f32::MAX;
    let mut best_normal = Vector3f::ZERO;

    let project_tri = |axis: Vector3f| -> (f32, f32) {
        let mut mn = f32::MAX;
        let mut mx = f32::MIN;
        for &v in &tri.verts {
            let p = v.dot(&axis);
            if p < mn { mn = p; }
            if p > mx { mx = p; }
        }
        (mn, mx)
    };

    let project_obb = |axis: Vector3f| -> (f32, f32) {
        let c = obb.center.dot(&axis);
        let h = obb_project_half(obb, &axis);
        (c - h, c + h)
    };

    let mut test = |axis: Vector3f| -> bool {
        let len_sq = axis.length_squared();
        if len_sq < 1e-10 { return true; }
        let axis = axis / len_sq.sqrt();

        let (mn_t, mx_t) = project_tri(axis);
        let (mn_b, mx_b) = project_obb(axis);
        let overlap = mx_t.min(mx_b) - mn_t.max(mn_b);
        if overlap < 0.0 { return false; } // gap

        if overlap < min_depth {
            min_depth = overlap;
            let obb_center_proj = obb.center.dot(&axis);
            let tri_center_proj = (tri.verts[0] + tri.verts[1] + tri.verts[2]).dot(&axis) / 3.0;
            best_normal = if obb_center_proj > tri_center_proj { axis } else { -axis };
        }
        true
    };

    if !test(tri.normal) {
        return None;
    }

    for i in 0..3 {
        if !test(obb.axes[i]) {
            return None;
        }
    }

    let edges = [
        tri.verts[1] - tri.verts[0],
        tri.verts[2] - tri.verts[1],
        tri.verts[0] - tri.verts[2]
    ];
    for &e in &edges {
        for i in 0..3 {
            if !test(e.cross(&obb.axes[i])) {
                return None;
            }
        }
    }

    let point = closest_obb_point_to_plane(obb, &tri.normal, &tri.verts[0]);
    Some(Contact { normal: best_normal, depth: min_depth, point })
}

fn closest_obb_point_to_plane(obb: &WorldBox, plane_normal: &Vector3f, plane_point: &Vector3f) -> Vector3f {
    let he = obb.half_extents;
    let signs = [
        [-1.0_f32, -1.0, -1.0], [-1.0, -1.0, 1.0],
        [-1.0,  1.0, -1.0], [-1.0,  1.0, 1.0],
        [ 1.0, -1.0, -1.0], [ 1.0, -1.0, 1.0],
        [ 1.0,  1.0, -1.0], [ 1.0,  1.0, 1.0]
    ];
    let mut best_pt = obb.center;
    let mut best_dist = f32::MAX;
    for s in &signs {
        let corner = obb.center
            + obb.axes[0] * (he.x * s[0])
            + obb.axes[1] * (he.y * s[1])
            + obb.axes[2] * (he.z * s[2]);
        let dist = (corner - plane_point).dot(plane_normal).abs();
        if dist < best_dist { best_dist = dist; best_pt = corner; }
    }
    best_pt
}

fn triangle_mesh_obb(tm: &WorldTriangleMesh, obb: &WorldBox) -> Option<Contact> {
    let obb_aabb = obb.aabb();
    let candidates = tm.query_aabb(&obb_aabb);

    let mut best: Option<Contact> = None;
    for idx in candidates {
        let tri = &tm.triangles[idx];
        if let Some(contact) = triangle_vs_obb(tri, obb) {
            let keep = best.as_ref().map_or(true, |b| contact.depth < b.depth);
            if keep { best = Some(contact); }
        }
    }
    best
}

fn triangle_mesh_sphere(tm: &WorldTriangleMesh, s: &WorldSphere) -> Option<Contact> {
    let sphere_aabb = s.aabb();
    let candidates  = tm.query_aabb(&sphere_aabb);

    let mut best: Option<Contact> = None;
    for idx in candidates {
        let tri = &tm.triangles[idx];
        let cp = closest_point_on_triangle(s.center, tri.verts[0], tri.verts[1], tri.verts[2]);
        let diff    = s.center - cp;
        let dist_sq = diff.length_squared();
        if dist_sq >= s.radius * s.radius { continue; }
        let dist   = dist_sq.sqrt();
        let normal = if dist < 1e-6 { tri.normal } else { diff / dist };
        let depth  = s.radius - dist;
        let keep = best.as_ref().map_or(true, |b: &Contact| depth < b.depth);
        if keep {
            best = Some(Contact { normal, depth, point: cp });
        }
    }
    best
}

fn triangle_mesh_capsule(tm: &WorldTriangleMesh, cap: &WorldCapsule) -> Option<Contact> {
    let cap_aabb   = cap.aabb();
    let candidates = tm.query_aabb(&cap_aabb);

    let mut best: Option<Contact> = None;
    for idx in candidates {
        let tri = &tm.triangles[idx];
        let cp_tri = closest_point_on_triangle(
            (cap.tip_a + cap.tip_b) * 0.5,
            tri.verts[0], tri.verts[1], tri.verts[2],
        );
        let (cp_seg, _) = closest_point_on_segment(cap.tip_a, &cap.tip_b, &cp_tri);
        let cp_tri2 = closest_point_on_triangle(cp_seg, tri.verts[0], tri.verts[1], tri.verts[2]);

        let diff    = cp_seg - cp_tri2;
        let dist_sq = diff.length_squared();
        if dist_sq >= cap.radius * cap.radius { continue; }
        let dist   = dist_sq.sqrt();
        let normal = if dist < 1e-6 { tri.normal } else { diff / dist };
        let depth  = cap.radius - dist;
        let keep = best.as_ref().map_or(true, |b: &Contact| depth < b.depth);
        if keep {
            best = Some(Contact { normal, depth, point: cp_tri2 });
        }
    }
    best
}

fn closest_point_on_triangle(p: Vector3f, a: Vector3f, b: Vector3f, c: Vector3f) -> Vector3f {
    let ab = b - a; let ac = c - a; let ap = p - a;
    let d1 = ab.dot(&ap); let d2 = ac.dot(&ap);
    if d1 <= 0.0 && d2 <= 0.0 { return a; }

    let bp = p - b;
    let d3 = ab.dot(&bp); let d4 = ac.dot(&bp);
    if d3 >= 0.0 && d4 <= d3 { return b; }

    let cp = p - c;
    let d5 = ab.dot(&cp); let d6 = ac.dot(&cp);
    if d6 >= 0.0 && d5 <= d6 { return c; }

    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return a + ab * v;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return a + ac * w;
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return b + (c - b) * w;
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    a + ab * v + ac * w
}

fn project_mesh(mesh: &WorldConvexMesh, axis: &Vector3f) -> (f32, f32) {
    let mut mn =  f32::MAX;
    let mut mx = f32::MIN;
    for &v in &mesh.vertices {
        let p = v.dot(axis);
        if p < mn { mn = p; }
        if p > mx { mx = p; }
    }
    (mn, mx)
}