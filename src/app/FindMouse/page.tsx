"use client";

import React, { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import styles from "./FindMouse.module.css"; // 引入 CSS 模块

const FindMouse = () => {
  type FindMousePayload = {
    indicator_x: number;
    indicator_y: number;
    window_width: number;
    window_height: number;
  };
  const animationRef = useRef<HTMLDivElement>(null);
  const [findmousePayload, setFindMousePayload] = useState<FindMousePayload>({
    indicator_x: 0,
    indicator_y: 0,
    window_width: 0,
    window_height: 0,
  });

  useEffect(() => {
    let animationId = 0;
    let findMouseTimeoutId: number | any = 0;

    const handleFindMouse = () => {
      cancelAnimation(); // 如果之前有动画在运行，先取消
      startAnimation(); // 开始新的动画
    };

    const startAnimation = () => {
      let startTime: DOMHighResTimeStamp = 0;
      const animate = (timestamp: DOMHighResTimeStamp) => {
        if (!startTime) {
          startTime = timestamp;
        }
        const elapsed_time = timestamp - startTime;

        // 更新动画状态
        updateAnimation(elapsed_time);
        // 继续请求下一帧动画
        animationId = requestAnimationFrame(animate);
      };

      animationId = requestAnimationFrame(animate);
    };

    const cancelAnimation = () => {
      updateAnimation(0);
      cancelAnimationFrame(animationId);
    };

    const updateAnimation = (elapsed_time: number) => {
      const circle = animationRef.current;
      if (circle) {
        circle.style.transform = `translate(-50%, -50%) scale(${((elapsed_time * 5) % 500) / 500})`;
      }
    };

    listen("find_mouse", (event: { payload: FindMousePayload }) => {
      // 如果之前的隐藏鼠标位置提示的超时回调还没有调用就清除掉，新建一个重新计时
      setFindMousePayload({
        indicator_x: event.payload.indicator_x,
        indicator_y: event.payload.indicator_y,
        window_width: event.payload.window_width,
        window_height: event.payload.window_height,
      });
      if (findMouseTimeoutId) {
        clearTimeout(findMouseTimeoutId);
      }
      // 设置一定时间后隐藏鼠标位置的提示
      findMouseTimeoutId = setTimeout(async () => {
        const window = await import("@tauri-apps/api/window");
        await window.getCurrent().hide();
        findMouseTimeoutId = null;
        cancelAnimation();
      }, 500);

      handleFindMouse();
    });
    return () => {
      cancelAnimation(); // 在组件卸载时取消动画
    };
  }, []);

  return (
    <>
      <style jsx global>{`
        body {
          margin: 0;
          padding: 0;
          box-sizing: border-box;
          overflow: hidden;
        }
        * {
          margin: 0;
          padding: 0;
          box-sizing: border-box;
          overflow: hidden;
        }
      `}</style>
      <div
        className={styles.findmouse_background}
        style={{
          width: findmousePayload.window_width,
          height: findmousePayload.window_height,
        }}
      >
        <div
          style={{
            left: findmousePayload.indicator_x,
            top: findmousePayload.indicator_y,
          }}>
        </div>
        <div
          ref={animationRef}
          className={styles.circle}
          style={{
            left: findmousePayload.indicator_x,
            top: findmousePayload.indicator_y,
          }}
        />
        {`x: ${findmousePayload.indicator_x} y: ${findmousePayload.indicator_y}`}
      </div>
    </>
  );
};

export default FindMouse;
