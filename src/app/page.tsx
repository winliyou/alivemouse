'use client';
import { KeyboardEvent, use, useEffect, useState } from 'react';
import React from 'react';
import { emit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { Container, TextField, Grid, Typography, styled, Link } from '@mui/material';


function getValidKey(key: string) {
  // 使用正则表达式检查key是否匹配
  if (/^[a-z]$/i.test(key)) {
    return key.toUpperCase();
  } else if (/^Key[A-Z]$/.test(key)) {
    return key.slice(3).toUpperCase();
  } else if (/^Digit[0-9]$/.test(key)) {
    return key.slice(5);
  } else {
    return null;
  }
}

function ShortcutKeyInput(props: { initialValue: string, onSave: (value: string) => void }) {
  const [value, setValue] = useState(props.initialValue);
  const [error, setError] = useState(false);
  const [ctrlPressed, setCtrlPressed] = useState(false);
  const [shiftPressed, setShiftPressed] = useState(false);
  const [altPressed, setAltPressed] = useState(false);
  const [normalkey, setNormalkey] = useState("");

  useEffect(() => {
    setValue(props.initialValue);
  }, [props.initialValue]);

  const handleKeyDown = (event: KeyboardEvent) => {
    let ctrlPressed_ = ctrlPressed;
    let shiftPressed_ = shiftPressed;
    let altPressed_ = altPressed;
    if (event.key === "Control") {
      setCtrlPressed(true);
      ctrlPressed_ = true;
    }
    if (event.key === "Shift") {
      setShiftPressed(true);
      shiftPressed_ = true;
    }
    if (event.key === "Alt") {
      setAltPressed(true);
      altPressed_ = true;
    }
    console.log(
      `ctrlpressed_: ${ctrlPressed_} , altPressed_: ${altPressed_}, shiftPressed_: ${shiftPressed_},
      ctrlpressed: ${ctrlPressed} , altPressed: ${altPressed}, shiftPressed: ${shiftPressed}`,
    );
    let newValue = "";
    if (ctrlPressed_) newValue += newValue ? "+Ctrl" : "Ctrl";
    if (shiftPressed_) newValue += newValue ? "+Shift" : "Shift";
    if (altPressed_) newValue += newValue ? "+Alt" : "Alt";
    if (
      event.key !== "Control" &&
      event.key !== "Shift" &&
      event.key !== "Alt"
    ) {
      console.log("code: ", event.code);
      let normalKey = getValidKey(event.code);
      if (normalKey != null) {
        newValue += newValue ? `+${normalKey}` : normalKey;
      }
    }
    setValue(newValue);
  };

  const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key === "Control") {
      setCtrlPressed(false);
    }
    if (event.key === "Shift") {
      setShiftPressed(false);
    }
    if (event.key === "Alt") {
      setAltPressed(false);
    }
  };

  const handleBlur = () => {
    const isValidShortcut = /^((ctrl|alt|shift)\+)*[a-z0-9]$/i.test(value);
    if (isValidShortcut) {
      setError(false);
      if (props.onSave) {
        props.onSave(value);
      }
    } else {
      setValue("");
      setError(true);
    }
  };

  return (
    <TextField
      error={error}
      id="outlined-error-helper-text"
      label="快捷键"
      value={value}
      onKeyDown={handleKeyDown}
      onKeyUp={handleKeyUp}
      onBlur={handleBlur}
      helperText={error ? "无效的快捷键" : ""}
      variant="outlined"
    />
  );
}

export default () => {
  const [mouseMoveInterval, setMouseMoveInterval] = useState<number>(5);
  const [hotkey, setHotKey] = useState<string>("ctrl+alt+p");
  useEffect(() => {
    (async () => {
      const default_mouse_move_interval = await invoke('get_move_mouse_interval');
      const default_hotkey = await invoke('get_find_mouse_hotkey');
      setMouseMoveInterval(default_mouse_move_interval as number);
      setHotKey(default_hotkey as string);
    })();
  }, []);

  const handleSubmit = (event: React.FormEvent) => {
    event.preventDefault();
    emit('move_mouse_interval_change', { interval: mouseMoveInterval });
    emit('find_mouse_hotkey_change', { hotkey: hotkey });
  };
  return (
    <Container maxWidth="md" style={{ height: '100vh', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <Grid container spacing={3}>
        <Grid item xs={12}>
          <Typography variant="h4" component="h1" gutterBottom>
            设置
          </Typography>
        </Grid>
        <Grid item xs={12}>
          <TextField
            fullWidth
            label="闲置时间(s)"
            value={`${mouseMoveInterval}`}
            type="number"
            InputLabelProps={{
              shrink: true,
            }}
            variant="outlined"
            onBlur={(e) => {
              emit('move_mouse_interval_change', { interval: mouseMoveInterval });
            }}
            onChange={(e) => {
              setMouseMoveInterval(parseInt(e.target.value));
            }}
          />
        </Grid>
        <Grid item xs={12}>
          <ShortcutKeyInput initialValue={hotkey} onSave={(value: string) => {
            setHotKey(value);
            emit('find_mouse_hotkey_change', { hotkey: value });
          }} />
        </Grid>
      </Grid>
    </Container>
  );
}