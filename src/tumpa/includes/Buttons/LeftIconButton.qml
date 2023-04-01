import QtQuick

Rectangle {
    id: root
    signal clicked
    property bool active: false
    property alias imageSource: keyIcon.source
    property alias text: keymgmtTxt.text

    width: parent.width
    height: 40
    color: active ? "#45187E" : "#54298B"
    radius: 5

    // Makes icon items keyboard focusable and pressable
    activeFocusOnTab: true
    border.width: root.activeFocus ? 2 : 0
    border.color: root.activeFocus ? "#CCA4FF" : "transparent"
    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Space || event.key === Qt.Key_Enter) {
                            root.clicked()
                        }
                    }

    Row {
        spacing: 8
        anchors.leftMargin: 8
        anchors.rightMargin: 8
        anchors.topMargin: 8
        anchors.bottomMargin: 8
        anchors.fill: parent

        Image {
            id: keyIcon
            source: "../../images/key_icon.svg"
            anchors.verticalCenter: parent.verticalCenter
        }

        Text {
            id: keymgmtTxt
            text: qsTr("")
            color: "white"
            anchors.verticalCenter: parent.verticalCenter
            font.pixelSize: 14
            font.weight: 500
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
