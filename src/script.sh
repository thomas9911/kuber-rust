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

list() {
    echo "command1"
    echo "command2"
    echo "command3"
    echo "command4"
    echo "command5"
    echo "command6"
    echo "command7"
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

ctx)
    kubectx_wrapper "$@"
    ;;

*)
    print_help
    ;;
esac
