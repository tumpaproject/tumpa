import QtQuick

Row {
    property alias statusText: status.text

    id: root
    height: 24
    spacing: 8

    Image {
        id: icon
        source: "../../images/card_status.svg"
        anchors.verticalCenter: parent.verticalCenter
        width: 24
        height: 24
        fillMode: Image.PreserveAspectFit
        sourceSize.width: 1024
        sourceSize.height: 1024
        mipmap: true
    }

    Text {
        id: status
        text: qsTr("")
        color: "#CCA4FF"
        font.pixelSize: 12
        font.weight: 400
        lineHeight: 1.25
        anchors.verticalCenter: parent.verticalCenter
    }
}
