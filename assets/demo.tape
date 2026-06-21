# holdout demo — a verifier you cannot game
# Regenerate from repo root:  cargo build --release && vhs assets/demo.tape
Output assets/demo.gif
Set Shell "bash"
Set FontSize 14
Set Width 1320
Set Height 760
Set Padding 14
Set TypingSpeed 42ms

Hide
Type "export PATH=$PWD/target/release:$PATH PS1='$ '"
Enter
Type "clear"
Enter
Show

Sleep 0.4s
Type "# holdout: grade a candidate against a trusted reference on FRESH inputs it never saw"
Enter
Sleep 1.2s

# A faithful refactor (different syntax, same behavior) passes.
Type `holdout verify --reference 'awk {print($1*$1)}' --candidate 'awk {print($1^2)}' --generator 'seq 1 50'`
Enter
Sleep 2.2s

Type "# now a candidate that looks plausible but is wrong:"
Enter
Sleep 1s
Type `holdout verify --reference 'awk {print($1*$1)}' --candidate 'awk {print($1+$1)}' --generator 'seq 1 50'`
Enter
Sleep 2.6s

Type "# caught on a held-out input — exit 1, with the first divergence localized."
Enter
Sleep 2.5s
