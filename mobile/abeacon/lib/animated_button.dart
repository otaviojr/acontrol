import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/widgets.dart';

class AnimatedButtonWidget extends StatefulWidget {
  final bool animated;
  final double? width;
  final double? height;
  final TextSpan buttonText;

  final Duration reversePlusDuration;
  final double reversePlusDistance;

  final Duration plusDuration;
  final double plusDistance;
  final double borderThickness;

  final Color buttonColor;
  final Function? onButtonTap;

  AnimatedButtonWidget({
    this.animated = false,
    Key? key,
    this.width,
    this.height,
    this.reversePlusDistance = 20,
    this.onButtonTap,
    required this.buttonText,
    this.plusDuration = const Duration(milliseconds: 300),
    this.buttonColor = const Color(0xFF0099ff),
    this.plusDistance = 4,
    this.borderThickness = 6,
    this.reversePlusDuration = const Duration(milliseconds: 1500),
  }) : super(key: key);

  @override
  AnimatedButtonWidgetState createState() =>
      AnimatedButtonWidgetState();
}

class AnimatedButtonWidgetState
    extends State<AnimatedButtonWidget>
    with TickerProviderStateMixin {

  ///
  ///
  ///Animation controller for go button render box
  late AnimationController plusAnimationController;
  late AnimationController reversePlusAnimationController;
  late Timer t;

  @override
  void initState() {
    plusAnimationController =
        AnimationController(vsync: this, duration: widget.plusDuration);
    reversePlusAnimationController =
        AnimationController(vsync: this, duration: widget.reversePlusDuration);

    plusAnimationController.addListener(() {
      if (plusAnimationController.isCompleted) {
        plusAnimationController.reverse();
      }
      if (plusAnimationController.isDismissed && widget.animated) {
        reversePlusAnimationController.forward();
        t = Timer(const Duration(seconds: 3), () {
          plusAnimationController.forward();
        });
      }
    });

    reversePlusAnimationController.addListener(() {
      if (reversePlusAnimationController.isCompleted) {
        reversePlusAnimationController.reset();
      }
    });

    super.initState();
  }

  @override
  void didUpdateWidget(AnimatedButtonWidget oldWidget) {
    if(oldWidget.animated != widget.animated){
      if(widget.animated){
        plusAnimationController.forward();
      } else {
        plusAnimationController.stop();
        reversePlusAnimationController.stop();
        t?.cancel();
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      child: Ink(
          decoration: BoxDecoration( shape: BoxShape.circle),
          child: InkWell(
            onTap: () {
              if (widget.onButtonTap != null) {
                widget.onButtonTap!();
              }
            },
            child: AnimatedButtonWidgetObject(
              state: this,
            ),
            customBorder: const CircleBorder(),
          )
      ),
    );
  }
}

class AnimatedButtonWidgetObject extends LeafRenderObjectWidget {
  AnimatedButtonWidgetObject({
    required this.state,
  });

  @override
  RenderObject createRenderObject(BuildContext context) {
    return AnimatedButtonWidgetRenderBox(state: state);
  }

  final AnimatedButtonWidgetState state;

  @override
  void updateRenderObject(BuildContext context,
      covariant AnimatedButtonWidgetRenderBox renderObject) {}
}

class AnimatedButtonWidgetRenderBox extends RenderBox {
  final AnimatedButtonWidgetState state;
  Animation plusAnimation;

  AnimatedButtonWidgetRenderBox({
    required this.state,
  }) : plusAnimation = CurvedAnimation(
      parent: state.plusAnimationController, curve: Curves.ease);

  @override
  Size computeDryLayout(BoxConstraints constraints) {
    return Size(
      state.widget.width ??
          (constraints.hasBoundedWidth ? constraints.maxWidth : 140),
      state.widget.height ??
          (constraints.hasBoundedHeight ? constraints.maxHeight : 140),
    );
  }

  @override
  bool get sizedByParent => true;

  Path? path;
  Path? closePathFormGradient;

  @override
  void paint(PaintingContext context, Offset offset) {
    Canvas canvas = context.canvas;

    canvas.translate(offset.dx, offset.dy);

    Rect rect = Rect.fromCenter(
        center: Offset(size.width / 2, size.height / 2),
        width: size.width,
        height: size.height);

    double buttonRadius = size.width / 2 - state.widget.reversePlusDistance;

    Paint paint = Paint()
      ..color = state.widget.buttonColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = state.widget.borderThickness;

    canvas.drawCircle(rect.center,
        buttonRadius - state.widget.plusDistance * plusAnimation.value, paint);

    if (state.reversePlusAnimationController.isAnimating) {
      canvas.drawCircle(
          rect.center,
          buttonRadius +
              state.widget.reversePlusDistance *
                  state.reversePlusAnimationController.value,
          paint
            ..color = state.widget.buttonColor
                .withOpacity(1 - state.reversePlusAnimationController.value));
    }

    final textPainter = TextPainter(
      text: state.widget.buttonText,
      textDirection: TextDirection.ltr,
    );
    textPainter.layout(
      minWidth: 0,
      maxWidth: size.width,
    );

    final xCenter = (textPainter.width) / 2;
    final yCenter = (textPainter.height) / 2;
    final textCenterOffset = Offset(xCenter, yCenter);
    textPainter.paint(canvas, rect.center - textCenterOffset);
  }

  @override
  void attach(PipelineOwner owner) {
    super.attach(owner);
    state.plusAnimationController.addListener(markNeedsPaint);
    state.reversePlusAnimationController.addListener(markNeedsPaint);
  }

  @override
  void detach() {
    state.plusAnimationController.removeListener(markNeedsPaint);
    state.reversePlusAnimationController.removeListener(markNeedsPaint);
    super.detach();
  }

// @override
// bool hitTestSelf(Offset position) => true;
}