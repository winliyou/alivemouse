'use client';
import { useState } from 'react';
import React from 'react';
import { emit } from '@tauri-apps/api/event';

const App = () => {

  const [mouseMoveInterval, setMouseMoveInterval] = useState(10);
  const [hotkey, setHotKey] = useState('Ctrl+Alt+P');

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    emit('move_mouse_interval_change', { interval: mouseMoveInterval });
    emit('find_mouse_hotkey_change', { hotkey: hotkey });
  };


  return (
    <div>
      <form onSubmit={handleSubmit}>
        <div>
          <label>
            移动鼠标时间间隔:
            <input type="number" value={mouseMoveInterval} onChange={e => setMouseMoveInterval(parseInt(e.target.value))} min="1" />
          </label>
        </div>
        <div>
          <label>
            显示鼠标位置的快捷键
            <input type="text" value={hotkey} onChange={e => setHotKey(e.target.value)} />
          </label>
        </div>
        <button type="submit">保存</button>
      </form>
    </div>
  );
};

export default App;