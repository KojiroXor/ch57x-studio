use ch57x_compiler::allocator::allocate_triggers;
use ch57x_compiler::diff::DeploymentDecision;
use ch57x_compiler::yaml_gen::generate_yaml;
use ch57x_domain::actions::{ActionId, HidAction, HidMod, MediaToken, MouseToken, SlotBinding};
use ch57x_domain::profile::Profile;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[test]
fn test_full_54_slots_validation_against_upstream_binary() {
    let mut profile = Profile::default();

    // Configura i 54 slot con una varietà realistica di binding:
    // Slot 0..11: Tasti Layer 1 (HostActionRef con trigger fantasma)
    for i in 0..12 {
        profile.slots.get_mut(&i).unwrap().binding = SlotBinding::HostActionRef(ActionId::new_v4());
    }

    // Slot 12..14: Knob 0 Layer 1 (Media e mouse)
    profile.slots.get_mut(&12).unwrap().binding =
        SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::VolumeDown));
    profile.slots.get_mut(&13).unwrap().binding =
        SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::Mute));
    profile.slots.get_mut(&14).unwrap().binding =
        SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::VolumeUp));

    // Slot 15..17: Knob 1 Layer 1 (Mouse wheel e click)
    profile.slots.get_mut(&15).unwrap().binding =
        SlotBinding::HidNative(HidAction::MouseAction(MouseToken::WheelUp));
    profile.slots.get_mut(&16).unwrap().binding =
        SlotBinding::HidNative(HidAction::MouseAction(MouseToken::Click));
    profile.slots.get_mut(&17).unwrap().binding =
        SlotBinding::HidNative(HidAction::MouseAction(MouseToken::WheelDown));

    // Slot 18..29: Tasti Layer 2 (Scorciatoie HID native)
    for i in 18..24 {
        profile.slots.get_mut(&i).unwrap().binding =
            SlotBinding::HidNative(HidAction::KeyboardChord {
                modifiers: vec![HidMod::Ctrl],
                key: format!("{}", (b'a' + (i - 18) as u8) as char),
            });
    }
    for i in 24..30 {
        profile.slots.get_mut(&i).unwrap().binding = SlotBinding::HostActionRef(ActionId::new_v4());
    }

    // Slot 30..35: Knobs Layer 2 (HostActionRef per knob)
    for i in 30..36 {
        profile.slots.get_mut(&i).unwrap().binding = SlotBinding::HostActionRef(ActionId::new_v4());
    }

    // Slot 36..47: Tasti Layer 3 (Mix Disabled e HostActionRef)
    for i in 36..42 {
        profile.slots.get_mut(&i).unwrap().binding = SlotBinding::Disabled;
    }
    for i in 42..48 {
        profile.slots.get_mut(&i).unwrap().binding = SlotBinding::HostActionRef(ActionId::new_v4());
    }

    // Slot 48..53: Knobs Layer 3 (Media actions)
    profile.slots.get_mut(&48).unwrap().binding =
        SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::PrevSong));
    profile.slots.get_mut(&49).unwrap().binding =
        SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::PlayPause));
    profile.slots.get_mut(&50).unwrap().binding =
        SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::NextSong));
    profile.slots.get_mut(&51).unwrap().binding =
        SlotBinding::HidNative(HidAction::ConsumerMedia(MediaToken::Stop));
    profile.slots.get_mut(&52).unwrap().binding =
        SlotBinding::HidNative(HidAction::MouseAction(MouseToken::RightClick));
    profile.slots.get_mut(&53).unwrap().binding =
        SlotBinding::HidNative(HidAction::MouseAction(MouseToken::MiddleClick));

    // 1. Alloca i trigger
    allocate_triggers(&mut profile).expect("Trigger allocation must succeed");

    // 2. Genera il documento YAML
    let yaml = generate_yaml(&profile).expect("YAML generation must succeed");
    assert!(!yaml.is_empty());

    // 3. Verifica hash e diff
    let decision = DeploymentDecision::new(&yaml, None);
    assert!(decision.requires_hardware_flash);

    // 4. Se il binario ch57x-keyboard-tool esiste, esegui la validazione reale
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bin_path = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("bin")
        .join("ch57x-keyboard-tool.exe");

    if bin_path.exists() {
        let mut child = Command::new(&bin_path)
            .arg("validate")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn ch57x-keyboard-tool.exe");

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(yaml.as_bytes())
                .expect("Failed to write YAML to stdin");
        }

        let output = child
            .wait_with_output()
            .expect("Failed to wait on validation process");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        println!("CLI validate stdout: {}", stdout);
        println!("CLI validate stderr: {}", stderr);

        assert!(
            output.status.success(),
            "Validation failed! Output: {}\nStderr: {}",
            stdout,
            stderr
        );
        assert!(
            stdout.contains("config is valid"),
            "Expected 'config is valid' in output, got: {}",
            stdout
        );
    }
}
