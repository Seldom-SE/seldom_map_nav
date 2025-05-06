//! Navmesh structures and generation

#[cfg(feature = "bevy")]
use crate::prelude::*;
use crate::vertex::{Ordinal, VertexNormal};

#[cfg(feature = "bevy")]
use bevy_platform::collections::HashMap;
use cdt::triangulate_with_edges;
use glam::{UVec2, Vec2};
use mint::Vector3;
use navmesh::NavMesh;
#[cfg(not(feature = "bevy"))]
use std::collections::HashMap;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

#[derive(Clone, Debug)]
struct NavmeshEntry {
    navmesh: NavMesh,
    clearance: f32,
}

/// Put this component on your tilemap. Stores your map's navmeshes.
#[cfg_attr(feature = "bevy", derive(Component))]
#[derive(Clone, Debug)]
pub struct Navmeshes(Vec<NavmeshEntry>);

impl Navmeshes {
    /// Generate navmeshes for your tilemap. The input to `navability` is a tile's position.
    /// `clearances` will be sorted for you.
    pub fn generate(
        map_size: UVec2,
        tile_size: Vec2,
        navability: impl Fn(UVec2) -> Navability,
        clearances: impl IntoIterator<Item = f32>,
    ) -> Result<Self, NavmeshGenError> {
        let mut clearances = clearances.into_iter().collect::<Vec<_>>();
        clearances.sort_by(f32::total_cmp);

        let mut navmeshes = Vec::with_capacity(clearances.len());
        for clearance in clearances {
            navmeshes.push(NavmeshEntry {
                navmesh: generate_navmesh(map_size, tile_size, &navability, clearance)?,
                clearance,
            });
        }

        Ok(Self(navmeshes))
    }

    /// Gets the navmesh with the least amount of clearance
    /// greater than or equal to the given clearance
    pub fn mesh(&self, clearance: f32) -> Option<&NavMesh> {
        let Navmeshes(navmeshes) = self;
        navmeshes
            .get(navmeshes.partition_point(|navmesh| clearance > navmesh.clearance))
            .map(|navmesh| &navmesh.navmesh)
    }

    /// Gets a navmesh at the given index. Navmeshes are sorted from least to most clearance.
    pub fn mesh_at(&self, mesh: usize) -> Option<&NavMesh> {
        let Navmeshes(navmeshes) = self;
        navmeshes.get(mesh).map(|entry| &entry.navmesh)
    }

    /// Gets the number of navmeshes
    pub fn mesh_count(&self) -> usize {
        let Navmeshes(navmeshes) = self;
        navmeshes.len()
    }
}

/// Represents the conditions under which this tile is navigable. More variants
/// should be added in the future, as breaking changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Navability {
    /// This tile can be navigated
    Navable,
    /// This tile cannot be navigated and navigators should avoid colliding with it
    Solid,
}

/// Error that can emit when generating a navmesh
#[derive(Debug)]
pub enum NavmeshGenError {
    /// Error related to generating triangles for the navmesh
    Triangulation(cdt::Error),
    /// Error related to constructing the navmesh from triangles
    Navmesh(navmesh::Error),
}

impl Display for NavmeshGenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Triangulation(error) => format!("{error}"),
                Self::Navmesh(navmesh::Error::TriangleVerticeIndexOutOfBounds(
                    triangle,
                    local_vertex,
                    global_vertex,
                )) => format!(
                    "vertex {local_vertex} {global_vertex} in triangle {triangle} is out of bounds"
                ),
                Self::Navmesh(navmesh::Error::ConnectionVerticeIndexOutOfBounds(
                    connection,
                    local_vertex,
                    global_vertex,
                )) => format!(
                    "vertex {local_vertex} {global_vertex} in connection {connection} is out of bounds"
                ),
                Self::Navmesh(navmesh::Error::CouldNotSerializeNavMesh(error)) =>
                    format!("could not serialize navmesh: {error}"),
                Self::Navmesh(navmesh::Error::CouldNotDeserializeNavMesh(error)) =>
                    format!("could not deserialize navmesh: {error}"),
                Self::Navmesh(navmesh::Error::CellsCountDoesNotMatchColsRows(
                    cell_count,
                    column_count,
                    row_count,
                )) => format!(
                    "{cell_count} cell count conflicts with {row_count} rows and {column_count} columns"
                ),
                Self::Navmesh(navmesh::Error::EmptyCells(column_count, row_count)) => format!(
                    "row or column count is 0 (rows: {row_count}, columns: {column_count})"
                ),
                Self::Navmesh(navmesh::Error::InvalidCellCoordinate(
                    column,
                    row,
                    column_count,
                    row_count,
                )) => format!(
                    "invalid cell coordinate at row {row} column {column}; there are {row_count} rows and {column_count} columns"
                ),
            }
        )
    }
}

impl Error for NavmeshGenError {}

impl From<cdt::Error> for NavmeshGenError {
    fn from(error: cdt::Error) -> Self {
        NavmeshGenError::Triangulation(error)
    }
}

impl From<navmesh::Error> for NavmeshGenError {
    fn from(error: navmesh::Error) -> Self {
        NavmeshGenError::Navmesh(error)
    }
}

/// Generate a navmesh for your tilemap. The input to `navability` is a tile's position.
pub fn generate_navmesh(
    map_size: UVec2,
    tile_size: Vec2,
    navability: impl Fn(UVec2) -> Navability,
    clearance: f32,
) -> Result<NavMesh, NavmeshGenError> {
    let mut vertex_normals =
        vec![VertexNormal::None; ((map_size.x + 1) * (map_size.y + 1)) as usize];
    let vertex_index = |x, y| (y * (map_size.x + 1) + x) as usize;

    for x in 0..=map_size.x {
        vertex_normals[vertex_index(x, 0)].add_assn(Ordinal::Northeast);
        vertex_normals[vertex_index(x, 0)].add_assn(Ordinal::Northwest);
        vertex_normals[vertex_index(x, map_size.y)].add_assn(Ordinal::Southeast);
        vertex_normals[vertex_index(x, map_size.y)].add_assn(Ordinal::Southwest);
    }

    for y in 0..map_size.y {
        vertex_normals[vertex_index(0, y)].add_assn(Ordinal::Southeast);
        vertex_normals[vertex_index(0, y + 1)].add_assn(Ordinal::Northeast);
        vertex_normals[vertex_index(map_size.x, y)].add_assn(Ordinal::Southwest);
        vertex_normals[vertex_index(map_size.x, y + 1)].add_assn(Ordinal::Northwest);
    }

    let navability = &navability;
    let navability = (0..map_size.y)
        .flat_map(|y| (0..map_size.x).map(move |x| navability(UVec2::new(x, y))))
        .collect::<Vec<_>>();

    for y in 0..map_size.y {
        for x in 0..map_size.x {
            if navability[(y * map_size.x + x) as usize] == Navability::Solid {
                vertex_normals[vertex_index(x, y)].add_assn(Ordinal::Southwest);
                vertex_normals[vertex_index(x + 1, y)].add_assn(Ordinal::Southeast);
                vertex_normals[vertex_index(x, y + 1)].add_assn(Ordinal::Northwest);
                vertex_normals[vertex_index(x + 1, y + 1)].add_assn(Ordinal::Northeast);
            }
        }
    }

    let mut vertices = Vec::default();
    let mut vert_edge_parts = HashMap::<(u32, bool), usize>::default();
    let mut horz_edge_parts = HashMap::<(u32, bool), usize>::default();
    let mut edges = Vec::default();

    for y in 0..=map_size.y {
        for x in 0..=map_size.x {
            let normal = vertex_normals[vertex_index(x, y)];

            for (ordinal, inner) in normal.normals() {
                let index = vertices.len();

                vertices
                    .push(UVec2::new(x, y).as_vec2() * tile_size + ordinal.as_vec2() * clearance);

                let (north, east) = ordinal.parts();
                let (vert_start, horz_start) = match inner {
                    true => (north, east),
                    false => (!north, !east),
                };

                if vert_start {
                    vert_edge_parts.insert((x, east), index);
                } else {
                    edges.push((vert_edge_parts.remove(&(x, east)).unwrap(), index));
                }

                if horz_start {
                    horz_edge_parts.insert((y, north), index);
                } else {
                    edges.push((horz_edge_parts.remove(&(y, north)).unwrap(), index));
                }
            }
        }
    }

    Ok(NavMesh::new(
        vertices
            .iter()
            .map(|vertex| Vector3::from(vertex.extend(0.)).into())
            .collect(),
        triangulate_with_edges(
            &vertices
                .iter()
                .map(|vertex| (vertex.x as f64, vertex.y as f64))
                .collect::<Vec<_>>(),
            &edges,
        )?
        .into_iter()
        .filter_map(|(v1, v2, v3)| {
            let tile = ((vertices[v1] + vertices[v2] + vertices[v3]) / 3. / tile_size).as_uvec2();
            (navability[(tile.y * map_size.x + tile.x) as usize] == Navability::Navable)
                .then(|| (v1 as u32, v2 as u32, v3 as u32).into())
        })
        .collect(),
    )?)
}
