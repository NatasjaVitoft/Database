import { useState } from "react";

export function Document() {

    return (
        <>
            <h4>Document name</h4>
            <div className="document-container">
                <textarea name="document-text" id="document-text"></textarea>
            </div>
        </>
    );
}
