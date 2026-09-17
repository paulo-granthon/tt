_tt() {
    local cur prev words cword
    _init_completion 2>/dev/null || {
        cur="${COMP_WORDS[COMP_CWORD]}"
        prev="${COMP_WORDS[COMP_CWORD-1]}"
    }

    local langs="auto en pt-BR pt-PT es fr de it ja ko zh-CN zh-TW ru ar nl sv pl tr hi el he cs da fi no uk vi th id ro hu"
    local flags="-q --quiet -s --synonyms -v --verbose -j --json --no-cache -h --help"

    case "$cur" in
        sl=*) COMPREPLY=($(compgen -W "$langs" -P "sl=" -- "${cur#sl=}")); return ;;
        tl=*) COMPREPLY=($(compgen -W "$langs" -P "tl=" -- "${cur#tl=}")); return ;;
        f=*)  COMPREPLY=($(compgen -f -P "f=" -- "${cur#f=}")); return ;;
        -*)   COMPREPLY=($(compgen -W "$flags" -- "$cur")); return ;;
    esac

    if [ "${COMP_CWORD}" -eq 1 ]; then
        COMPREPLY=($(compgen -W "default profile languages cache update sl= tl= p= f= $flags" -- "$cur"))
        return
    fi

    case "${COMP_WORDS[1]}" in
        profile)
            if [ "${COMP_CWORD}" -eq 2 ]; then
                COMPREPLY=($(compgen -W "add list ls delete rm patch" -- "$cur"))
            else
                COMPREPLY=($(compgen -W "sl= tl= name=" -- "$cur"))
            fi
            return ;;
        default)
            COMPREPLY=($(compgen -W "sl= tl=" -- "$cur")); return ;;
        cache)
            COMPREPLY=($(compgen -W "clear" -- "$cur")); return ;;
    esac

    COMPREPLY=($(compgen -W "sl= tl= p= f= $flags" -- "$cur"))
}
complete -F _tt tt
