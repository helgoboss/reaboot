import {lazy} from 'solid-js';
import type {RouteDefinition} from '@solidjs/router';

import Install from './pages/install';
import Faq from './pages/faq';
import {Share} from "./pages/share";
import Home from "./pages/home";

export const routes: RouteDefinition[] = [
    {
        path: '/',
        component: Home,
    },
    {
        path: '/install/:thing',
        component: Install,
    },
    {
        path: '/faq',
        component: Faq,
    },
    {
        path: '/share',
        component: Share,
    },
    {
        path: '**',
        component: lazy(() => import('./errors/404')),
    },
];
