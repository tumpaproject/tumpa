import QtQuick

Rectangle {
    id: root
    signal clicked
    property bool active: false
    property alias imageSource: keyIcon.source

    width: parent.width
    height: 40
    anchors.topMargin: 45
    color: active ? "#45187E" : "#54298B"
    radius: 5

    Row {
        spacing: 11
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        anchors.topMargin: 8
        anchors.bottomMargin: 8
        anchors.fill: parent

        Image {
            id: keyIcon
            source: "../../images/key_icon.svg"
            anchors.verticalCenter: keymgmtTxt.verticalCenter
        }

        Text {
            id: keymgmtTxt
            text: qsTr("Key Management")
            color: "white"
        }
    }
    MouseArea {
        id: mArea
        anchors.fill: parent
        onClicked: {
            root.clicked()
        }
    }
}
