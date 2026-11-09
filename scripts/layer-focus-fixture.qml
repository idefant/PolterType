// Run only inside an isolated nested Wayland session.
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import QtQuick

ShellRoot {
    FloatingWindow {
        visible: true
        implicitWidth: 600
        implicitHeight: 250
        title: "PolterType layer-focus fixture"
        TextInput { id: editor; anchors.fill: parent; focus: true }
    }
    PanelWindow {
        id: menu
        visible: false
        anchors { top: true; bottom: true; left: true; right: true }
        WlrLayershell.namespace: "poltertype-test-modal"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
        TextInput { id: search; anchors.fill: parent; focus: true }
    }
    PanelWindow {
        id: passive
        visible: false
        implicitWidth: 200
        implicitHeight: 50
        anchors.top: true
        WlrLayershell.namespace: "poltertype-test-passive"
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None
    }
    IpcHandler {
        target: "poltertype_e2e"
        function openMenu(): void { search.text = ""; menu.visible = true; search.forceActiveFocus() }
        function closeMenu(): void { menu.visible = false; editor.forceActiveFocus() }
        function menuText(): string { return search.text }
        function editorText(): string { return editor.text }
        function resetEditor(): void { menu.visible = false; passive.visible = false; editor.text = ""; editor.forceActiveFocus() }
        function showPassive(): void { passive.visible = true }
    }
}
