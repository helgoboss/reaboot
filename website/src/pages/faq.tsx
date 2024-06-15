import {Page} from "../components/page";

export default function Faq() {
    return (
        <Page>
            <div class="h-responsive-prose">
                <h1 class="text-center">Frequently Asked Questions</h1>
                <dl>
                    <dt>
                        What is ReaBoot?
                    </dt>
                    <dd>
                        ReaBoot is a convenient all-in-one online installer for REAPER, ReaPack and arbitrary packages.
                        You probably ended up here because someone wanted to give you an easy way to install REAPER
                        scripts or extensions.
                    </dd>
                    <dt>
                        I'm afraid that ReaBoot creates dirty installations. Is it safer to install REAPER and ReaPack
                        by
                        hand?
                    </dt>
                    <dd>
                        It's ReaBoot's highest priority to create installations that are not different from what you
                        would get by installing everything by hand. The result should be exactly the same. Nice and
                        clean.
                        If you discover an exception, please report it as a bug!
                    </dd>
                    <dt>
                        Can I use this to update existing REAPER installations?
                    </dt>
                    <dd>
                        Absolutely. You can use it over and over again in order to add new packages or replace existing
                        ones.
                    </dd>
                    <dt>
                        Is ReaBoot a replacement for ReaPack?
                    </dt>
                    <dd>
                        Not at all! ReaBoot embraces ReaPack and installs all packages in a way that's 100% compatible
                        with it. The main idea is to use ReaBoot to make the initial installation easier. After that,
                        you can use ReaPack to install new packages. Or you can continue using ReaBoot. The choice is
                        yours! You can even use them interchangeably, it shouldn't make any difference.
                    </dd>
                    <dt>
                        Why does ReaBoot always install the ReaPack extension?
                    </dt>
                    <dd>
                        Technically, ReaBoot doesn't need ReaPack to install or replace packages. But ReaPack is
                        important
                        for you at a later stage, for example in order to:
                        <ul>
                            <li>Browse and install packages directly from within REAPER</li>
                            <li>Remove packages</li>
                            <li>Automatically register installed ReaScripts</li>
                        </ul>
                    </dd>
                    <dt>
                        Can I use ReaBoot while REAPER is running?
                    </dt>
                    <dd>
                        Yes. But it might ask you to quit REAPER if it turns out to be an issue.
                    </dd>
                    <dt>
                        Do I lock myself in by relying on ReaBoot installation links?
                    </dt>
                    <dd>
                        No. All parts of ReaBoot are open-source (GPL-3) and will remain so, including the website
                        you are currently seeing.
                    </dd>
                    <dt>
                        I would like to share not just packages but also apply user settings. Is that supported by
                        ReaBoot?
                    </dt>
                    <dd>
                        No. Neither ReaPack nor ReaBoot touch user settings, because this would quickly lead to
                        conflicts.
                        Please consider creating, zipping and sharing a portable REAPER installation instead.
                    </dd>
                </dl>
            </div>
        </Page>
    );
}
