/* @refresh reload */
import './index.css';
import 'prism-material-themes/themes/material-ocean.css';

import {render} from 'solid-js/web';
import {Router} from '@solidjs/router';
import {routes} from "./routes";
import {App} from "./app";
import {ErrorBoundary} from "solid-js";

const root = document.getElementById('root')!;

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
    throw new Error(
        'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
    );
}

render(
    () => (
        <ErrorBoundary fallback={(err, reset) => <div onClick={reset}>Error: {err.toString()}</div>}>
            <Router root={App}>
                {routes}
            </Router>
        </ErrorBoundary>
    ),
    root,
);
