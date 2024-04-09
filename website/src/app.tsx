import {JSX} from 'solid-js';
import {A, RouteSectionProps} from '@solidjs/router';

import {FaBrandsGithub} from "solid-icons/fa";
import {Welcome} from "./components/welcome";

export function App(props: RouteSectionProps) {
    return props.children;
}