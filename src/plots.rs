pub mod extraction;
pub mod plot_registry;

mod plot_errors {
    use std::{error, fmt};

    #[derive(Debug, Clone)]
    pub struct PlotError;

    impl fmt::Display for PlotError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "invalid first item to double")
        }
    }

    impl error::Error for PlotError {}
}

pub mod plot_utils {
    pub mod resolution {
        pub const R720: (u32, u32) = (1280, 720);
        pub const R1080: (u32, u32) = (1920, 1080);
        pub const R4K: (u32, u32) = (3840, 2160);
    }

    pub mod palettes {
        /*
        Useful links:
        create palettes: https://coolors.co/fe5f55-f0b67f-d6d1b1-c7efcf-eef5db
        expand palettes: https://mycolor.space/?hex=%23D6D1B1&sub=1
        from hex to rgb: https://www.rapidtables.com/convert/color/hex-to-rgb.html
        */
        use plotters::style::RGBAColor;

        pub struct Palette {
            pub background: RGBAColor,
            pub mesh: RGBAColor,
            pub colors: [RGBAColor; 20],
        }
        pub const RED_PALETTE: Palette = Palette {
            background: RGBAColor(248, 247, 241, 1.0),
            mesh: RGBAColor(200, 200, 200, 1.0),
            colors: [
                RGBAColor(109, 118, 152, 1.0),
                RGBAColor(185, 186, 163, 1.0),
                RGBAColor(214, 213, 201, 1.0),
                RGBAColor(162, 44, 41, 1.0),
                RGBAColor(148, 83, 35, 1.0),
                RGBAColor(85, 68, 115, 1.0),
                RGBAColor(123, 150, 224, 1.0),
                RGBAColor(151, 42, 80, 1.0),
                RGBAColor(187, 120, 110, 1.0),
                RGBAColor(109, 118, 152, 1.0),
                RGBAColor(172, 99, 170, 1.0),
                RGBAColor(56, 99, 0, 1.0),
                RGBAColor(209, 231, 224, 1.0),
                RGBAColor(97, 168, 255, 1.0),
                RGBAColor(170, 107, 112, 1.0),
                RGBAColor(252, 133, 178, 1.0),
                RGBAColor(0, 86, 178, 1.0),
                RGBAColor(168, 174, 156, 1.0),
                RGBAColor(255, 120, 106, 1.0),
                RGBAColor(137, 114, 110, 1.0),
            ],
        };
        /* pub const BLUE_PALETTE: Palette = Palette {
            background: RGBAColor(255, 255, 255, 1.0),
            mesh: RGBAColor(128, 128, 128, 1.0),
            colors: [
                RGBAColor(9, 36, 39, 1.0),
                RGBAColor(11, 83, 81, 1.0),
                RGBAColor(0, 169, 185, 1.0),
                RGBAColor(78, 128, 152, 1.0),
                RGBAColor(144, 194, 231, 1.0),
                RGBAColor(121, 124, 177, 1.0),
                RGBAColor(67, 153, 110, 1.0),
                RGBAColor(149, 177, 175, 1.0),
                RGBAColor(113, 95, 88, 1.0),
                RGBAColor(61, 115, 154, 1.0),
            ],
        };
        pub const PASTEL_PALETTE: Palette = Palette {
            background: RGBAColor(255, 255, 255, 1.0),
            mesh: RGBAColor(128, 128, 128, 1.0),
            colors: [
                RGBAColor(254, 95, 85, 1.0),
                RGBAColor(240, 182, 127, 1.0),
                RGBAColor(214, 209, 177, 1.0),
                RGBAColor(199, 239, 207, 1.0),
                RGBAColor(238, 245, 219, 1.0),
                RGBAColor(225, 146, 136, 1.0),
                RGBAColor(182, 129, 77, 1.0),
                RGBAColor(0, 131, 81, 1.0),
                RGBAColor(185, 168, 154, 1.0),
                RGBAColor(159, 155, 12, 1.0),
            ],
        }; */
    }
}
