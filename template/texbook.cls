\NeedsTeXFormat{LaTeX2e}

\ProvidesClass{texbook}
\LoadClass[twoside,openright,12pt]{report}

\RequirePackage{../../template/ztxbase}

% Link to documents.tex for external references between notes.
\ifx\HCode\UnDeFiNeD
    \subimport{../../notes/}{documents.tex}
\fi

\newcommand{\transclude}[2][note]{\ExecuteMetaData[../../notes/slipbox/#2.tex]{#1}}

% Newline fix in catchfilebetweentags
\RequirePackage{etoolbox}
\makeatletter
\patchcmd{\CatchFBT@Fin@l}{\endlinechar\m@ne}{}
{}{\typeout{Unsuccessful patch!}}
\makeatother

% References between notes (keeps real links in HTML)
\makeatletter
\newcommand{\exhyperref}[3][note]{%
    \@ifundefined{r@#1}%
        {% External reference
            \ifx\HCode\UnDeFiNeD
                \hyperref[#2-#1]{#3}%
            \else
                \ztxhtmlhref{#2}{#2-#1}{#3}%
            \fi
        }%
        {% Local reference (same behavior)
            \hyperref[#1]{#3}%
        }%
}

\newcommand{\excref}[2][note]{%
    \@ifundefined{r@#1}%
        {% External reference
            \ifthenelse{\equal{#1}{note}}
                {\ifx\HCode\UnDeFiNeD
                    \hyperref[#2-#1]{\texttt{#2}}%
                \else
                    \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                \fi}%
                {\ifx\HCode\UnDeFiNeD
                    \exhyperref[#1]{#2}{\texttt{#2/}\cref*{#2-#1}}%
                \else
                    \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                \fi}%
        }%
        {% Local reference
            \ifthenelse{\equal{#1}{note}}
                {\cref{#1}}
                {\cref{#1}}%
        }%
}

\newcommand{\exref}[2][note]{%
    \@ifundefined{r@#1}%
        {% External reference
            \ifthenelse{\equal{#1}{note}}
                {\ifx\HCode\UnDeFiNeD
                    \hyperref[#2-#1]{\texttt{#2}}%
                \else
                    \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                \fi}%
                {\ifx\HCode\UnDeFiNeD
                    \exhyperref[#1]{#2}{\texttt{#2/}\ref*{#2-#1}}%
                \else
                    \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                \fi}%
        }%
        {% Local reference
            \ifthenelse{\equal{#1}{note}}
                {\ref{#1}}
                {\ref{#1}}%
        }%
}
\makeatother

\ProcessOptions