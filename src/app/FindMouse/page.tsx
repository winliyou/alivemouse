'use client';
import "../globals.css";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react"
export default () => {
    const [estinate, setEstimate] = useState(2);
    const [currentTimeout, setCurrentTimeout] = useState<any>(null);
    useEffect(() => {
        listen("find_mouse", (event) => {
            if (currentTimeout) {
                clearTimeout(currentTimeout);
            }
            setCurrentTimeout(setTimeout(async () => {
                const window = await import("@tauri-apps/api/window");
                window.getCurrent().hide().then(() => {
                    console.log("hide from web")
                });
            }, 500))
        })
    }, [])
    return (
        <>
            <div className="circle"></div>
        </>
    );
}