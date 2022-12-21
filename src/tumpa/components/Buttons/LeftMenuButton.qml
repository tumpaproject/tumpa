import QtQuick

Rectangle {
    id: root
    signal clicked
    property bool active: false
    property alias text: keyTxt.text

    width: parent.width
    height: 40
    color: active ? "#45187E" : "#54298B"
    radius: 5

    Text {
        id: keyTxt
        anchors {
            fill: parent
            margins: 10
            leftMargin: 40
        }
        text: qsTr("")
        color: "#DED4E9"
        font.pixelSize: 14
        font.weight: 500
    }

    MouseArea {
        id: mArea
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}
