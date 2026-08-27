//! The embedded faces, and measuring type with them.

use breadify::font::{ALL, Face};
use breadify::geometry::{PRODUCT_COLUMN_WIDTH, mm_to_pt, pt_to_mm};
use breadify::text::{self, Style};

#[test]
fn every_face_is_a_static_instance_of_the_family_it_claims() {
    for face in ALL {
        let parsed = face.parsed();

        assert!(
            !parsed.is_variable(),
            "{:?} is a variable font — it would embed its default instance silently",
            face
        );

        let family = parsed
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FAMILY)
            .and_then(|name| name.to_string())
            .unwrap_or_default();
        assert!(
            family.starts_with(face.family()),
            "{face:?} claims {} but the file says {family:?}",
            face.family()
        );
    }
}

#[test]
fn the_weights_are_the_ones_the_design_asks_for() {
    let weights: Vec<u16> = ALL.iter().map(|face| face.weight()).collect();
    assert_eq!(weights, [800, 900, 400, 500, 400, 500, 600, 700]);
}

#[test]
fn the_longest_product_name_fits_its_column_at_the_eleven_point_floor() {
    let longest = "Holdbart Havrebrød Skåret 750g Bakehuset (har Vært Fryst)";
    assert_eq!(longest.chars().count(), 57);

    let width = text::width(longest, Style::new(Face::SpaceGrotesk, 11.0));
    assert!(
        width < PRODUCT_COLUMN_WIDTH,
        "the longest name needs {width:.1} mm of a {PRODUCT_COLUMN_WIDTH} mm column"
    );
    assert!(
        width > 100.0,
        "a {width:.1} mm measurement means the face is not being read"
    );
}

#[test]
fn norwegian_letters_are_in_every_face() {
    for face in ALL {
        let parsed = face.parsed();
        for character in ['æ', 'ø', 'å', 'Æ', 'Ø', 'Å'] {
            assert!(
                parsed.glyph_index(character).is_some(),
                "{face:?} has no {character}"
            );
        }
    }
}

#[test]
fn tracking_widens_a_run_and_the_mono_face_is_monospaced() {
    let plain = Style::new(Face::MonoRegular, 8.0);
    let tracked = plain.tracked(0.10);
    assert!(text::width("PICKED", tracked) > text::width("PICKED", plain));

    let one = text::width("M", plain);
    let two = text::width("MM", plain);
    let ten = text::width("iiiiiiiiii", plain);
    assert!(
        (two - 2.0 * one).abs() < 1e-9,
        "mono advances should be equal"
    );
    assert!((ten - 10.0 * one).abs() < 1e-9);
}

#[test]
fn millimetres_and_points_convert_exactly_one_way_and_back() {
    assert!((pt_to_mm(72.0) - 25.4).abs() < 1e-12);
    assert!((mm_to_pt(210.0) - 595.275_590_551_181).abs() < 1e-9);
    assert!((mm_to_pt(pt_to_mm(11.0)) - 11.0).abs() < 1e-12);
}

#[test]
fn an_empty_run_has_no_width() {
    assert_eq!(text::width("", Style::new(Face::SpaceGrotesk, 11.0)), 0.0);
}

#[test]
fn report_the_widest_measurements() {
    let longest = "Holdbart Havrebrød Skåret 750g Bakehuset (har Vært Fryst)";
    println!(
        "product name 11pt: {:.2} mm",
        text::width(longest, Style::new(Face::SpaceGrotesk, 11.0))
    );
    println!(
        "customer 14pt: {:.2} mm",
        text::width(
            "Customer 112",
            Style::new(Face::ArchivoExtraBold, 14.0).tracked(-0.02)
        )
    );
    println!(
        "department 11.9pt: {:.2} mm",
        text::width(
            "Department 10",
            Style::new(Face::ArchivoExtraBold, 11.9).tracked(-0.015)
        )
    );
}
