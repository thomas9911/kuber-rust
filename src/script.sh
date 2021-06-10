#! /bin/bash

# no dependencies test drop-in script for kuber

print_help() {
    echo "help text"
}

kubectx_wrapper() {
    if [ -n "$1" ]; then
        echo "use $1"
    else
        echo "test1"
        echo "test2"
        echo "test3"
    fi
}


apps() {
    echo "apps1"
    echo "apps2"
    echo "apps3"
    echo "apps4"
    echo "apps5"
    echo "apps6"
    echo "apps7"
    echo "apps8"
}

list() {
    if [ -n "$1" ]; then
        echo "command -> $1"
    else
        echo "command1"
        echo "command2"
        echo "command3"
        echo "command4"
        echo "command5"
        echo "command6"
        echo "command7"
        echo "command8"
    fi
}

cmd=$1
shift 1

case $cmd in
help)
    print_help
    ;;

h)
    print_help
    ;;
"-h")
    print_help
    ;;
"--help")
    print_help
    ;;

list)
    list "$@"
    ;;

ls)
    list "$@"
    ;;

apps)
    apps "$@"
    ;;

ctx)
    kubectx_wrapper "$@"
    ;;

*)
    print_help
    ;;
esac
