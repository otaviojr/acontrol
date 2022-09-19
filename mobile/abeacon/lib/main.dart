import 'dart:async';

import 'package:flutter/material.dart';
import 'package:beacon_broadcast/beacon_broadcast.dart';
import 'package:permission_handler/permission_handler.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Flutter Demo',
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
      home: const MyHomePage(title: 'Flutter Demo Home Page'),
    );
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key, required this.title});

  // This widget is the home page of your application. It is stateful, meaning
  // that it has a State object (defined below) that contains fields that affect
  // how it looks.

  // This class is the configuration for the state. It holds the values (in this
  // case the title) provided by the parent (in this case the App widget) and
  // used by the build method of the State. Fields in a Widget subclass are
  // always marked "final".

  final String title;

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  static const String uuid = '39ED98FF-2900-441A-802F-9C398FC199D2';
  static const int majorId = 1;
  static const int minorId = 100;
  static const int transmissionPower = -59;
  static const String identifier = 'com.example.myDeviceRegion';
  static const AdvertiseMode advertiseMode = AdvertiseMode.lowPower;
  static const String layout = BeaconBroadcast.ALTBEACON_LAYOUT;
  static const int manufacturerId = 0x0118;
  static const List<int> extraData = [100];

  BeaconBroadcast beaconBroadcast = BeaconBroadcast();
  bool _isAdvertising = false;
  late BeaconStatus _isTransmissionSupported = BeaconStatus.notSupportedCannotGetAdvertiser;
  late StreamSubscription<bool> _isAdvertisingSubscription;

  int _counter = 0;

  @override
  void initState() {
    super.initState();
    beaconBroadcast
        .checkTransmissionSupported()
        .then((isTransmissionSupported) async {

            //Ask for runtime permissions if necessary.
            var status = await Permission.bluetoothAdvertise.request();
            if (status.isGranted) {
              setState(() {
                _isTransmissionSupported = isTransmissionSupported;
              });
            }
        });

    _isAdvertisingSubscription =
        beaconBroadcast.getAdvertisingStateChange().listen((isAdvertising) {
          setState(() {
            _isAdvertising = isAdvertising;
          });
        });
  }


  void _incrementCounter() {
    setState(() {
      // This call to setState tells the Flutter framework that something has
      // changed in this State, which causes it to rerun the build method below
      // so that the display can reflect the updated values. If we changed
      // _counter without calling setState(), then the build method would not be
      // called again, and so nothing would appear to happen.
      _counter++;
    });
  }

  @override
  Widget build(BuildContext context) {
    // This method is rerun every time setState is called, for instance as done
    // by the _incrementCounter method above.
    //
    // The Flutter framework has been optimized to make rerunning build methods
    // fast, so that you can just rebuild anything that needs updating rather
    // than having to individually change instances of widgets.
    return Scaffold(
      appBar: AppBar(
        // Here we take the value from the MyHomePage object that was created by
        // the App.build method, and use it to set our appbar title.
        title: const Text('Beacon Broadcast'),
      ),
      body: SingleChildScrollView(
        child: Padding(
          padding: const EdgeInsets.all(8.0),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text('Is transmission supported?',
                  style: Theme.of(context).textTheme.headline5),
              Text('$_isTransmissionSupported',
                  style: Theme.of(context).textTheme.subtitle1),
              Container(height: 16.0),
              Text('Has beacon started?',
                  style: Theme.of(context).textTheme.headline5),
              Text('$_isAdvertising',
                  style: Theme.of(context).textTheme.subtitle1),
              Container(height: 16.0),
              Center(
                child: ElevatedButton(
                  onPressed: () {
                    beaconBroadcast
                        .setUUID(uuid)
                        .setMajorId(majorId)
                        .setMinorId(minorId)
                        .setTransmissionPower(transmissionPower)
                        .setAdvertiseMode(advertiseMode)
                        .setIdentifier(identifier)
                        .setLayout(layout)
                        .setManufacturerId(manufacturerId)
                        .setExtraData(extraData)
                        .start();
                  },
                  child: Text('START'),
                ),
              ),
              Center(
                child: ElevatedButton(
                  onPressed: () {
                    beaconBroadcast.stop();
                  },
                  child: Text('STOP'),
                ),
              ),
              Text('Beacon Data',
                  style: Theme.of(context).textTheme.headline5),
              Text('UUID: $uuid'),
              Text('Major id: $majorId'),
              Text('Minor id: $minorId'),
              Text('Tx Power: $transmissionPower'),
              Text('Advertise Mode Value: $advertiseMode'),
              Text('Identifier: $identifier'),
              Text('Layout: $layout'),
              Text('Manufacturer Id: $manufacturerId'),
              Text('Extra data: $extraData'),
            ],
          ),
        ),
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _incrementCounter,
        tooltip: 'Increment',
        child: const Icon(Icons.add),
      ), // This trailing comma makes auto-formatting nicer for build methods.
    );
  }
}
