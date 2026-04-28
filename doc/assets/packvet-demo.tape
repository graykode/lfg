Set Shell bash
Set FontSize 24
Set Width 1400
Set Height 760
Set Padding 30
Set Margin 40
Set MarginFill "#f5f7fb"
Set BorderRadius 18
Set WindowBar Colorful
Set Theme "Catppuccin Mocha"
Set TypingSpeed 18ms

Hide
Type "export PS1='$ '"
Enter
Type "packvet() { printf '%s\n' 'packvet: checking npm i left-pad' 'packvet: resolving npm metadata for left-pad' 'packvet: preparing diff for left-pad 1.2.0 -> 1.3.0' 'packvet: reviewing diff with claude-cli' 'packvet: review passed for left-pad 1.3.0' 'packvet: CLAUDE: Benign minor version bump. Runtime code is unchanged.' 'packvet: evidence: docs, typings metadata, devDependency updates, and tests changed.' 'packvet: evidence: no lifecycle scripts, network access, postinstall hooks, or maintainer changes.' 'packvet: review log: ~/.packvet/reviews/reviews.jsonl' 'packvet: review completed for npm install. install was not executed.'; sleep 20; }"
Enter
Type "clear"
Enter
Show

Type "packvet review npm i left-pad"
Enter
Sleep 700ms

Screenshot doc/assets/packvet-demo.png
