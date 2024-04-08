import {lazy} from 'solid-js';
import type {RouteDefinition} from '@solidjs/router';

import Install from './pages/install';
import Faq from './pages/faq';

export const routes: RouteDefinition[] = [
    {
        path: '/install/:thing',
        component: Install,
    },
    {
        path: '/faq',
        component: Faq,
    },
    {
        path: '**',
        component: lazy(() => import('./errors/404')),
    },
];
