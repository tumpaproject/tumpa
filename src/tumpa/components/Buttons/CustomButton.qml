import QtQuick
import QtQuick.Controls

Button {
    required property string labelString
    property string iconSrc: ""
    property string buttonColor: "default"
    property bool isThin: false

    readonly property var colorMap: {
        "default": {
            background: "#E5E7EB",
            backgroundPressed: "#DADDE2",
            text: "#000000",
        },
        "green": {
            background: "#6EE7B7",
            backgroundPressed: "#66CC99",
            text: "#333333",
        },
        "white": {
            background: "#FFFFFF",
            backgroundPressed: "#FAFAFA",
            border: "#D1D5DB",
            text: "#333333",
        },
        "red": {
            background: "#B91C1C",
            backgroundPressed: "#FAFAFA",
            text: "#FFFFFF",
        },
        "red-alt": {
            background: "#FFFFFFF",
            backgroundPressed: "#FAFAFA",
            text: "#991B1B",
        }
    }

    id: control
    text: labelString
    font.weight: 500
    font.pixelSize: isThin ? 12 : 14

    contentItem: Text {
        text: control.text
        font: control.font
        opacity: enabled ? 1.0 : 0.3
        color: colorMap[buttonColor].text
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
        anchors.leftMargin: iconSrc ? 36 : 12
        anchors.left: parent.left
    }

    background: Rectangle {
        implicitWidth: implicitContentWidth + (iconSrc ? 48 : 24)
        implicitHeight: isThin ? 32 : 42
        opacity: enabled ? 1 : 0.3
        color: control.down ? colorMap[buttonColor].backgroundPressed : colorMap[buttonColor].background
        border.color: colorMap[buttonColor].border ? colorMap[buttonColor].border : transparent
        border.width: colorMap[buttonColor].border ? 1 : 0
        radius: 4
        Image {
            id: icon
            source: iconSrc
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            anchors.leftMargin: 12
            width: 12
            height: 9
            sourceSize.width: 1024
            sourceSize.height: 1024
        }
    }
}
