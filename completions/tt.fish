set -l tt_langs auto en pt-BR pt-PT es fr de it ja ko zh-CN zh-TW ru ar nl sv pl tr hi el he cs da fi no uk vi th id ro hu

complete -c tt -f

complete -c tt -n '__fish_use_subcommand' -a default -d 'get or set default languages'
complete -c tt -n '__fish_use_subcommand' -a profile -d 'manage profiles'
complete -c tt -n '__fish_use_subcommand' -a languages -d 'list supported language codes'
complete -c tt -n '__fish_use_subcommand' -a update -d 'install the latest release'

complete -c tt -n '__fish_seen_subcommand_from profile' -a 'add list ls delete rm patch'

complete -c tt -s q -l quiet -d 'print only the primary translation'
complete -c tt -s s -l synonyms -d 'print only the synonyms block'
complete -c tt -s v -l verbose -d 'labeled verbose breakdown'
complete -c tt -s j -l json -d 'full result as one JSON line'
complete -c tt -s h -l help -d 'show help'

complete -c tt -a 'f=' -d 'translate a file'
complete -c tt -n 'string match -q "f=*" -- (commandline -ct)' -f \
    -a '(printf "f=%s\n" (__fish_complete_path (string replace -r "^f=" "" -- (commandline -ct))))'

for lang in $tt_langs
    complete -c tt -a "sl=$lang" -d 'source language'
    complete -c tt -a "tl=$lang" -d 'target language'
end
