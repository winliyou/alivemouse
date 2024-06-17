import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';

const App = () => {
  useEffect(() => {
    // 监听窗口最小化事件
    listen('tauri://minimize', () => {
      invoke('hide_to_tray');
    });

    // 显示窗口（当应用程序准备好时）
    appWindow.hide();
  }, []);
  const [interval, setInterval] = useState(10);

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    invoke('set_interval', { interval: Number(interval) });
  };

  const handleStop = () => {
    invoke('pause_move');
  };

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <label>
          移动鼠标时间间隔:
          <input type="number" value={interval} onChange={e => setInterval(parseInt(e.target.value))} min="1" />
        </label>
        <button type="submit">保存</button>
      </form>
      <button onClick={handleStop}>停止移动</button>
    </div>
  );
};

export default App;