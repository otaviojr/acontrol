import 'dart:async';
import 'dart:io' show Platform;

import 'package:abeacon/animated_button.dart';
import 'package:flutter/material.dart';
import 'package:beacon_broadcast/beacon_broadcast.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:flutter_native_splash/flutter_native_splash.dart';

void main() {
  WidgetsBinding widgetsBinding = WidgetsFlutterBinding.ensureInitialized();
  FlutterNativeSplash.preserve(widgetsBinding: widgetsBinding);
  runApp(const ABeacon());
}

class ABeacon extends StatelessWidget {
  const ABeacon({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'ABeacon',
      theme: ThemeData(
        // This is the theme of your application.
        //
        // Try running your application with "flutter run". You'll see the
        // application has a blue toolbar. Then, without quitting the app, try
        // changing the primarySwatch below to Colors.green and then invoke
        // "hot reload" (press "r" in the console where you ran "flutter run",
        // or simply save your changes to "hot reload" in a Flutter IDE).
        // Notice that the counter didn't reset back to zero; the application
        // is not restarted.
        primarySwatch: Colors.blue,
      ),
      home: const ABeaconPage(title: 'ABeacon'),
    );
  }
}

class ABeaconPage extends StatefulWidget {
  const ABeaconPage({super.key, required this.title});

  // This widget is the home page of your application. It is stateful, meaning
  // that it has a State object (defined below) that contains fields that affect
  // how it looks.

  // This class is the configuration for the state. It holds the values (in this
  // case the title) provided by the parent (in this case the App widget) and
  // used by the build method of the State. Fields in a Widget subclass are
  // always marked "final".

  final String title;

  @override
  State<ABeaconPage> createState() => _MyABeaconPageState();
}

class _MyABeaconPageState extends State<ABeaconPage> {
  static const String uuid = '9BFBEF3A-210A-4B3A-9E58-24E7CD8354ED';
  static const int majorId = 1;
  static const int minorId = 100;
  static const int transmissionPower = -20;
  static const String identifier = 'com.otavioeng.acontrol';
  static const AdvertiseMode advertiseMode = AdvertiseMode.lowLatency;
  static const String layout = BeaconBroadcast.ALTBEACON_LAYOUT;
  static const int manufacturerId = 0x3712;
  static const List<int> extraData = [0x12];

  BeaconBroadcast beaconBroadcast = BeaconBroadcast();
  BeaconStatus _isTransmissionSupported = BeaconStatus.notSupportedCannotGetAdvertiser;
  late StreamSubscription<bool> _isAdvertisingSubscription;

  bool _isAdvertising = false;
  late Timer t;

  void _checkPermissions() async {
    var status = await Permission.bluetooth.request();

    Map<Permission, PermissionStatus> statuses = await [
      Permission.bluetooth,
      Permission.bluetoothAdvertise,
      Permission.location,
    ].request();

    /*if(await Permission.bluetooth.isDenied || await Permission.bluetooth.isPermanentlyDenied ||
      await Permission.bluetoothAdvertise.isDenied || await Permission.bluetoothAdvertise.isPermanentlyDenied ||
      await Permission.location.isDenied || await Permission.location.isPermanentlyDenied) {
      openAppSettings();
    }*/
  }

  @override
  void initState() {
    super.initState();

    _checkPermissions();

    beaconBroadcast
        .checkTransmissionSupported()
        .then((isTransmissionSupported) async {
            setState(() {
              _isTransmissionSupported = isTransmissionSupported;
            });
        });

    _isAdvertisingSubscription =
        beaconBroadcast.getAdvertisingStateChange().listen((isAdvertising) {
          setState(() {
            _isAdvertising = isAdvertising;
          });
        });

    Timer(const Duration(seconds: 2), () {
      FlutterNativeSplash.remove();
    });

  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        // Here we take the value from the MyHomePage object that was created by
        // the App.build method, and use it to set our appbar title.
        title: const Text('ABeacon Unlock'),
      ),
      body: Center(
        child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              /*Text('Is transmission supported?',
                  style: Theme.of(context).textTheme.headline5),
              Text('$_isTransmissionSupported',
                  style: Theme.of(context).textTheme.subtitle1),
              Container(height: 16.0),
              Text('Has beacon started?',
                  style: Theme.of(context).textTheme.headline5),
              Text('$_isAdvertising',
                  style: Theme.of(context).textTheme.subtitle1),
              Container(height: 16.0),*/
              AnimatedButtonWidget(
                animated: _isAdvertising,
                width: 220,
                height: 220,
                onButtonTap: () {
                  print("got button clicked");
                  if(!_isAdvertising) {
                    if (Platform.isAndroid) {
                      beaconBroadcast
                          .setUUID(uuid)
                          .setMajorId(majorId)
                          .setMinorId(minorId)
                          .setTransmissionPower(transmissionPower)
                          .setAdvertiseMode(advertiseMode)
                          .setLayout(layout)
                          .setManufacturerId(manufacturerId)
                          .setExtraData(extraData);
                    } else if (Platform.isIOS) {
                      beaconBroadcast
                          .setUUID(uuid)
                          .setMajorId(majorId)
                          .setMinorId(minorId)
                          .setIdentifier(identifier)
                          .setTransmissionPower(transmissionPower)
                          .setAdvertiseMode(advertiseMode);
                    }
                    beaconBroadcast.start();
                    t = Timer(const Duration(minutes: 10), () {
                      beaconBroadcast.stop();
                    });
                  } else {
                    t?.cancel();
                    beaconBroadcast.stop();
                  }
                },
                buttonText: TextSpan(
                    text: (_isAdvertising ? 'STOP' : 'GO'), style: TextStyle(color: Colors.black, fontSize: 32)),
              )
            ]
        ),
      ),
    );
  }
}
