import QtQuick
import QtQuick.Window
import QtQuick.Controls

import "includes/Forms"
import "includes/Buttons"
import "includes/Utils"

ApplicationWindow {
    title: qsTr("Tumpa")
    width: 900
    height: 551
    visible: true

    id: root

    // This defines if we will allow saving private key
    property bool allowsecret: false
    // For error message
    property string errortext: "Error text"

    ListModel {
        id: ksKeys
    }

    SplitView {
        anchors.fill: parent

        // Tags of the left side
        Rectangle {
            color: "#54298B"
            height: root.height
            SplitView.minimumWidth: 217

            Image {
                id: tumpaLogo
                source: "images/logo.svg"
                anchors {
                    topMargin: 14
                    top: parent.top
                    leftMargin: 14
                    left: parent.left
                }
            }

            Column {
                id: menuOptions
                anchors {
                    topMargin: 20
                    top: tumpaLogo.bottom
                    leftMargin: 14
                    left: parent.left
                    right: parent.right
                    rightMargin: 10
                }
                spacing: 4

                LeftIconButton {
                    id: leftIconBttn
                    // This is our initial active menu item
                    active: true
                    anchors.bottomMargin: 18
                    text: qsTr("Key Management")
                    onClicked: {
                        clearActive()
                        active = true
                        // FIXME: Show only if we have keys
                        stack.pop()
                        gotoKeyList()
                    }
                }

                ColSpacer {
                    height: 10
                }

                LeftIconButton {
                    id: leftKeyBttn
                    // This is our Yubikey button
                    imageSource: "../../images/usbkey.svg"
                    text: qsTr("Smart Card")
                    onClicked: {
                        clearActive()
                        active = true
                        console.log("leftKeyBttn")
                    }
                }

                LeftMenuButton {
                    id: editNameBttn
                    text: qsTr("Edit Name")
                    onClicked: {
                        clearActive()
                        active = true
                        // clear the stack first
                        //stack.pop()
                        stack.push(nameView)
                    }
                }

                LeftMenuButton {
                    id: editPublicURLBttn
                    text: qsTr("Edit Public URL")
                    onClicked: {
                        clearActive()
                        active = true
                        stack.push(publicurlView)
                    }
                }

                LeftMenuButton {
                    id: editUserPinBttn
                    text: qsTr("Change User Pin")
                    onClicked: {
                        clearActive()
                        active = true
                        stack.push(userpinView)
                    }
                }

                LeftMenuButton {
                    id: editAdminPinBttn
                    text: qsTr("Change Admin Pin")
                    onClicked: {
                        clearActive()
                        active = true
                        stack.push(adminpinView)
                    }
                }
            }
            StatusInfo {
                anchors {
                    bottom: parent.bottom
                    bottomMargin: 24
                    left: parent.left
                    leftMargin: 14
                }

                statusText: qsTr("Card detected")
            }
        }

        Rectangle {
            id: bigBox
            color: "white"
            height: root.height
            SplitView.minimumWidth: 683

            StackView {
                id: stack
                initialItem: keylistView
                anchors.fill: parent

                pushEnter: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 100
                    }
                }
                pushExit: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 1
                        to: 0
                        duration: 100
                    }
                }
                popEnter: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 0
                        to: 1
                        duration: 100
                    }
                }
                popExit: Transition {
                    PropertyAnimation {
                        property: "opacity"
                        from: 1
                        to: 0
                        duration: 100
                    }
                }
            }
        }
    }

    function clearActive() {
        for (var i in menuOptions.children) {
            if (!(menuOptions.children[i] instanceof ColSpacer)) {
                menuOptions.children[i].active = false
            }
        }
    }

    Connections {
        target: tbackend
        function onUpdated() {
            stack.pop()
            if (tbackend.haveKeys === true) {
                if (stack.depth === 1) {
                    stack.pop()
                }
                // Get the latest keys
                refreshKeyList()
                stack.push(keylistView)
            }
            //stack.push(ykView)
        }
    }

    Connections {
        target: tbackend
        function onRefreshKeys() {
            // Get the latest keys
            refreshKeyList()
        }
    }

    Component {
        id: startView
        StartView {
            //onClicked: stack.push(userView)
            onGenkeyclicked: stack.push(genkeyView)
        }
    }

    Component {
        id: genkeyView

        GenerateKeyView {
            onNext: {
                stack.pop()
                stack.push(waitView)
                tbackend.generateKey(name, emails, passphrase, expirationDate,
                                     encryptionChecked, signingChecked,
                                     authenticationChecked, keyAlgo)
            }
        }
    }

    Component {
        id: waitView
        WaitView {}
    }

    Component {
        id: keylistView
        KeyListView {
            keyListData: ksKeys
        }
    }

    Component {
        id: nameView
        NameView {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateName(name, adminpin)
                if (result === false) {
                    errortext = qsTr("Could not set the name in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    Component {
        id: publicurlView
        PublicURL {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateURL(url, adminpin)
                if (result === false) {
                    errortext = qsTr(
                                "Could not set the Public URL in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    Component {
        id: userpinView
        UserPin {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateUserPin(userpin, adminpin)
                if (result === false) {
                    errortext = qsTr(
                                "Could not set the User Pin in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    Component {
        id: adminpinView
        AdminPin {
            onNext: {
                // Let us have the logic here
                var result = tbackend.updateAdminPin(newpin, adminpin)
                if (result === false) {
                    errortext = qsTr(
                                "Could not set new Admin Pin in the Yubikey!")
                    var win = showErrorBox(qsTr("Error"), errortext)
                    return
                }
                // Else we go back
                stack.pop()
            }
        }
    }

    function showErrorBox(heading, errortext) {
        // Here we will show the modal dialog to show any error to the user
        var component1 = Qt.createComponent("includes/Utils/InfoModal.qml")
        var win = component1.createObject(root, {
                                              "headingText": heading,
                                              "contentText": errortext
                                          })
        win.okayed.connect(() => {
                               //console.log("Modal okayed")
                               win.destroy()
                           })
        win.show()
        return win
    }

    function refreshKeyList() {
        var localdata = tbackend.get_keys_json()
        //console.log(localdata)
        var data = JSON.parse(localdata)
        ksKeys.clear()
        for (var i in data) {
            ksKeys.append(data[i])
        }
    }

    function gotoKeyList() {
        if (tbackend.haveKeys === true) {
            if (stack.depth === 1) {
                stack.pop()
            }
            // Get the latest keys
            refreshKeyList()
            stack.push(keylistView)
        }
    }
}
