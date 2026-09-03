\NeedsTeXFormat{LaTeX2e}

\ProvidesClass{texnote}
\LoadClass[12pt]{article}

\RequirePackage{../../template/ztxbase}

% Link to documents.tex for external references between notes.
\ifx\HCode\UnDeFiNeD
    \subimport{../}{documents.tex}
\fi

\newcommand{\transclude}[2][note]{\ExecuteMetaData[#2.tex]{#1}}

% Automatically define the note's own anchor when calling \maketitle.
% Previously, each note file included the call (\currentdoc{note}); moving it
% here means it no longer needs to be written in every note. \currentdoc defines
% \label{note} and the {jobname}-note HTML anchors used by the "Referenced in"
% section.
\makeatletter
\let\ztxorig@maketitle\maketitle
\renewcommand{\maketitle}{%
    \ztxorig@maketitle
    \currentdoc{note}%
}
\makeatother

% In article there is no \chapter; it is styled as a bold title.
\newcommand{\chapter}[1]{\textbf{\LARGE#1}}

% References between notes
\makeatletter
\newcommand{\exhyperref}[3][note]{%
    \@ifundefined{r@#1}%
        {% External reference
            \ztxmaybehyperlink{%
                \ifx\HCode\UnDeFiNeD
                    \hyperref[#2-#1]{#3}%
                \else
                    \ztxhtmlhref{#2}{#2-#1}{#3}%
                \fi
            }{#3}%
        }%
        {% Local reference (same behavior)
            \ztxmaybehyperlink{\hyperref[#1]{#3}}{#3}%
        }%
}

\newcommand{\excref}[2][note]{%
    \@ifundefined{r@#1}%
        {% External reference
            \ifthenelse{\equal{#1}{note}}
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \hyperref[#2-#1]{\texttt{#2}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                    \fi
                }{\texttt{#2}}}%
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \exhyperref[#1]{#2}{\texttt{#2/}\cref*{#2-#1}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                    \fi
                }{%
                    \ifx\HCode\UnDeFiNeD
                        \texttt{#2/}\cref*{#2-#1}%
                    \else
                        \texttt{#2/#1}%
                    \fi
                }}%
        }%
        {% Local reference
            \ztxmaybehyperlink{%
                \ifthenelse{\equal{#1}{note}}
                    {\cref{#1}}
                    {\cref{#1}}%
            }{\ifthenelse{\equal{#1}{note}}{\cref{#1}}{\cref{#1}}}%
        }%
}

\newcommand{\exref}[2][note]{%
    \@ifundefined{r@#1}%
        {% External reference
            \ifthenelse{\equal{#1}{note}}
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \hyperref[#2-#1]{\texttt{#2}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2}}%
                    \fi
                }{\texttt{#2}}}%
                {\ztxmaybehyperlink{%
                    \ifx\HCode\UnDeFiNeD
                        \exhyperref[#1]{#2}{\texttt{#2/}\ref*{#2-#1}}%
                    \else
                        \ztxhtmlhref{#2}{#2-#1}{\texttt{#2/#1}}%
                    \fi
                }{%
                    \ifx\HCode\UnDeFiNeD
                        \texttt{#2/}\ref*{#2-#1}%
                    \else
                        \texttt{#2/#1}%
                    \fi
                }}%
        }%
        {% Local reference
            \ztxmaybehyperlink{%
                \ifthenelse{\equal{#1}{note}}
                    {\ref{#1}}
                    {\ref{#1}}%
            }{\ifthenelse{\equal{#1}{note}}{\ref{#1}}{\ref{#1}}}%
        }%
}

% In HTML references degrade to plain text.
\ifx\HCode\UnDeFiNeD
\else
\let\ztxorig@href\href
\renewcommand{\href}[2]{%
    \ztxmaybehyperlink{\ztxorig@href{#1}{#2}}{#2}%
}
\let\ztxorig@hyperref\hyperref
\renewcommand{\hyperref}[2][]{%
    \ztxmaybehyperlink{\ztxorig@hyperref[#1]{#2}}{#2}%
}
\fi
\makeatother

\ProcessOptions