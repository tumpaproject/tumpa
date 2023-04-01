import QtQuick

Row {
    property string keyName: ""
    property string valueName: ""

    spacing: 10

    Text {
        width: 175
        text: keyName
        color: "#6B7280"
        horizontalAlignment: Text.AlignRight
        font.pixelSize: 14
        font.weight: 500
    }
    Text {
        text: valueName
        color: "#111827"
        font.pixelSize: 14
        font.weight: 500
    }
}
