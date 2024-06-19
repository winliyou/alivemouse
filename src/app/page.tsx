'use client';
import { useState } from 'react';
import React from 'react';
import { invoke } from '@tauri-apps/api/tauri';

const App = () => {

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