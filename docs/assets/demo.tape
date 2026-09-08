# SkillPack 15-second demo: scaffold, assess, emit badge.
# Regenerate with: vhs docs/assets/demo.tape
# Requires a built skillpack binary. The tape tries the redirected target
# dir first (CARGO_TARGET_DIR=~/.cache/cargo-target), then ./target/release.

Output docs/assets/demo.gif

Set Shell "bash"
Set FontSize 16
Set Width 1000
Set Height 640
Set Padding 40

Type "export PATH=$HOME/.cache/cargo-target/release:$HOME/.cache/cargo-target/debug:$(pwd)/target/release:$PATH" Enter
Type "skillpack init demo-skill --template mcp" Enter
Sleep 1s
Type "cd demo-skill" Enter
Sleep 300ms
Type "skillpack grade ." Enter
Sleep 2s
Type "cd .." Enter
Sleep 300ms
Type "skillpack report demo-skill --format badge --output badge.svg" Enter
Sleep 1s
Type "echo rendered: && head -c 160 badge.svg" Enter
Sleep 1s
Type "rm -rf demo-skill badge.svg" Enter
