use fj_math::Vector;

use crate::objects::{Sketch, Solid, Stores};

use super::Sweep;

impl Sweep for Sketch {
    type Swept = Solid;

    fn sweep(self, path: impl Into<Vector<3>>, stores: &Stores) -> Self::Swept {
        let path = path.into();

        let mut shells = Vec::new();
        for face in self.into_faces() {
            let shell = face.sweep(path, stores);
            shells.push(shell);
        }

        Solid::new().with_shells(shells)
    }
}
