import QtQuick

Rectangle {
    id: root
    signal clicked
    property bool active: false
    property alias text: keyTxt.text

    width: parent.width
    height: 40
    anchors.topMargin: 45
    color: active ? "#45187E" : "#54298B"
    radius: 5

    Text {
        id: keyTxt
        anchors {
            fill: parent
            margins: 8
            leftMargin: 36
        }
        text: qsTr("Dummy text")
        color: "white"
    }

    MouseArea {
        id: mArea
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}
