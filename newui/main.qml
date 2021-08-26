import QtQuick 2.14
import QtQuick.Window 2.14
import QtQuick.Layouts 1.3
import QtQuick.Controls 2.5


ApplicationWindow {
    width: 800
    height: 600
    visible: true
    title: qsTr("Hello World")

    property int value1: 0
    property int value2: 0

    menuBar: MenuBar{

        Menu {
            title: "File"

            Action {
                id: newActionID
                text: "&New"
                // icon.source = "images/newFileIcon.png"
                onTriggered: {console.log("Clicked on New")}
            }

        }
    }

    TabBar {
        id: tabBar
        // TODO: add styling
        background: Rectangle {
            implicitHeight: 30

        }

        TabButton {
            id: keysButton
            text: "Available Keys"
            width: 200
            height: parent.height + 10
            font.weight: 20

        }

        TabButton {
            id: smartCardButton
            text: "Smart Card Options"
            width: 200
            height: parent.height + 10
            font.weight: 20
        }
    }

    StackLayout {
        id: mStackID
        anchors.top: tabBar.bottom
        width: parent.width
        height: 600

        //        anchors.centerIn: parent
        currentIndex: tabBar.currentIndex

        KeysDetails {
            id: keysTab
        }

        CardDetails {
            id: cardTab
        }

    }

}
