#!/bin/bash
# 🔥 VORTEX WORKSPACE CONTROL SCRIPT 🔥
# Generated for workspace: test-networking-v3

WORKSPACE_ID="test-networking-v3"
SERVICES=(frontend backend database cache)

start_workspace() {
    echo "🚀 Starting workspace: $WORKSPACE_ID"
    for service in "${SERVICES[@]}"; do
        vm_name="vortex-${WORKSPACE_ID}-$service"
        echo "   🔥 Starting $service ($vm_name)"
        DYLD_LIBRARY_PATH=/opt/homebrew/lib krunvm start "$vm_name"
    done
    echo "✅ Workspace started!"
}

stop_workspace() {
    echo "⏹️  Stopping workspace: $WORKSPACE_ID"
    for service in "${SERVICES[@]}"; do
        vm_name="vortex-${WORKSPACE_ID}-$service"
        echo "   🛑 Stopping $service ($vm_name)"
        DYLD_LIBRARY_PATH=/opt/homebrew/lib krunvm delete "$vm_name"
    done
    echo "✅ Workspace stopped!"
}

status_workspace() {
    echo "📊 Workspace status: $WORKSPACE_ID"
    DYLD_LIBRARY_PATH=/opt/homebrew/lib krunvm list | grep "vortex-$WORKSPACE_ID"
}

case "$1" in
    start)   start_workspace ;;
    stop)    stop_workspace ;;
    status)  status_workspace ;;
    restart) stop_workspace && sleep 2 && start_workspace ;;
    *)       echo "Usage: $0 {start|stop|status|restart}" ;;
esac
