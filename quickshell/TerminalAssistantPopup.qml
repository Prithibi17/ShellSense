import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland

// Minimal translucent suggestion popup styled for Caelestia / Quickshell
FloatingWindow {
    id: root
    visible: false
    color: "transparent"
    
    // Default popup dimensions
    width: 440
    height: contentColumn.implicitHeight + 24

    // Configurable properties
    property string suggestedCommand: ""
    property string commandDescription: ""
    property string riskLevel: "low" // "low", "medium", "destructive"
    property real confidence: 0.95

    Rectangle {
        id: bg
        anchors.fill: parent
        radius: 12
        color: "#E614161B" // Dark translucent surface
        border.color: riskLevel === "destructive" ? "#D9534F" : (riskLevel === "medium" ? "#E0AF68" : "#2A2E38")
        border.width: 1

        ColumnLayout {
            id: contentColumn
            anchors.fill: parent
            anchors.margins: 12
            spacing: 6

            // Header line
            RowLayout {
                Layout.fillWidth: true
                spacing: 8

                Rectangle {
                    width: 6
                    height: 6
                    radius: 3
                    color: riskLevel === "destructive" ? "#EF4444" : (riskLevel === "medium" ? "#F59E0B" : "#10B981")
                }

                Text {
                    text: riskLevel === "destructive" ? "⚠ Potentially Destructive Command" : "Suggested Command"
                    font.family: "JetBrains Mono, monospace"
                    font.pixelSize: 11
                    font.weight: Font.DemiBold
                    color: riskLevel === "destructive" ? "#EF4444" : "#9CA3AF"
                }

                Item { Layout.fillWidth: true }

                Text {
                    text: Math.round(root.confidence * 100) + "% match"
                    font.family: "JetBrains Mono, monospace"
                    font.pixelSize: 10
                    color: "#6B7280"
                }
            }

            // Command Display Box
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 34
                radius: 6
                color: "#1A1D24"
                border.color: "#252932"
                border.width: 1

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    spacing: 8

                    Text {
                        text: "❯"
                        font.family: "JetBrains Mono, monospace"
                        font.pixelSize: 12
                        color: "#60A5FA"
                    }

                    Text {
                        id: cmdText
                        text: root.suggestedCommand
                        font.family: "JetBrains Mono, monospace"
                        font.pixelSize: 12
                        font.weight: Font.Medium
                        color: "#F3F4F6"
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }
                }
            }

            // Description
            Text {
                text: root.commandDescription
                font.family: "Inter, sans-serif"
                font.pixelSize: 11
                color: "#9CA3AF"
                visible: text.length > 0
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
            }

            // Footer keybinding hints
            RowLayout {
                Layout.fillWidth: true
                Layout.topMargin: 4
                spacing: 12

                Text {
                    text: "[Tab / → Accept]"
                    font.family: "JetBrains Mono, monospace"
                    font.pixelSize: 10
                    color: "#60A5FA"
                }

                Text {
                    text: "[Esc Dismiss]"
                    font.family: "JetBrains Mono, monospace"
                    font.pixelSize: 10
                    color: "#6B7280"
                }

                Item { Layout.fillWidth: true }

                Text {
                    text: "terminal-assistant"
                    font.family: "JetBrains Mono, monospace"
                    font.pixelSize: 9
                    color: "#4B5563"
                }
            }
        }
    }

    // Public functions to control popup state from socket/IPC
    function showSuggestion(cmd, desc, risk, conf) {
        suggestedCommand = cmd;
        commandDescription = desc;
        riskLevel = risk;
        confidence = conf;
        visible = true;
    }

    function hideSuggestion() {
        visible = false;
    }
}
